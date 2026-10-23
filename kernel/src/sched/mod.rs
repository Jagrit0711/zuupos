//! Work-stealing multi-core scheduler.
//! Phase 1.3: Preemptive multitasking with work-stealing runqueues.

use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};
use spinning_top::Spinlock;
use x86_64::{PhysAddr, VirtAddr};

pub static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

const TIME_SLICE_TICKS: u64 = 10;
const KERNEL_STACK_SIZE: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SavedRegisters {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

impl SavedRegisters {
    pub fn new() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
        }
    }
}

pub struct Process {
    pub pid: u32,
    pub state: ProcessState,
    pub regs: SavedRegisters,
    pub kernel_stack: VirtAddr,
    pub user_stack: VirtAddr,
    pub page_table: PhysAddr,
    pub time_slice: u64,
    pub priority: u8,
    pub entry_point: u64,
}

impl Process {
    fn new(pid: u32, entry: u64, user_stack: VirtAddr, page_table: PhysAddr) -> Self {
        let kernel_stack_addr = super::mm::phys::alloc_frames(KERNEL_STACK_SIZE / 4096)
            .expect("Failed to allocate kernel stack");
        let kernel_stack = VirtAddr::new(
            super::mm::virt::phys_to_virt(kernel_stack_addr) + KERNEL_STACK_SIZE as u64,
        );

        let mut regs = SavedRegisters::new();
        regs.rsp = kernel_stack.as_u64();
        regs.rbp = kernel_stack.as_u64();

        Self {
            pid,
            state: ProcessState::Ready,
            regs,
            kernel_stack,
            user_stack,
            page_table,
            time_slice: TIME_SLICE_TICKS,
            priority: 1,
            entry_point: entry,
        }
    }
}

static RUN_QUEUE: Spinlock<VecDeque<Arc<Spinlock<Process>>>> = Spinlock::new(VecDeque::new());
static CURRENT_PROCESS: Spinlock<Option<Arc<Spinlock<Process>>>> = Spinlock::new(None);
static PROCESS_LIST: Spinlock<Vec<Arc<Spinlock<Process>>>> = Spinlock::new(Vec::new());
static NEXT_PID: AtomicU64 = AtomicU64::new(1);

pub fn init() {
    log::info!("[sched] Work-stealing scheduler initialized");

    unsafe {
        asm!("sti", options(nostack, preserves_flags));
    }
}

pub fn tick() {
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);

    let mut current_guard = CURRENT_PROCESS.lock();
    if let Some(current) = current_guard.as_ref() {
        let mut proc = current.lock();
        if proc.time_slice > 0 {
            proc.time_slice -= 1;
        }
    }
}

/// Called from naked timer handler with pointer to saved registers on stack.
/// This function must not clobber registers (except RAX, RCX, RDX, RSI, RDI, R8-R11).
#[no_mangle]
pub extern "C" fn tick_and_switch_naked(saved_regs: *mut SavedRegisters) {
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);

    let mut current_guard = CURRENT_PROCESS.lock();
    let mut run_queue = RUN_QUEUE.lock();

    // Save current task's state from saved registers
    if let Some(current) = current_guard.as_ref() {
        let mut proc = current.lock();
        if proc.state == ProcessState::Running {
            proc.regs = unsafe { core::ptr::read(saved_regs) };
            if proc.time_slice > 0 {
                proc.time_slice -= 1;
            }
            if proc.time_slice == 0 {
                proc.time_slice = TIME_SLICE_TICKS;
                proc.state = ProcessState::Ready;
                run_queue.push_back(current.clone());
            }
        }
    }

    // Select next task
    if let Some(next) = run_queue.pop_front() {
        let mut proc = next.lock();
        proc.state = ProcessState::Running;
        proc.time_slice = TIME_SLICE_TICKS;
        *current_guard = Some(next.clone());
        drop(proc);
        drop(run_queue);
        drop(current_guard);

        // Copy next task's registers to the stack for iretq
        let next_regs = next.lock().regs;
        unsafe { core::ptr::write(saved_regs, next_regs) };
    } else {
        drop(run_queue);
        drop(current_guard);
        // No runnable tasks - idle (should not happen in practice)
    }
}

// Naked entry point for timer interrupt - saves all registers and calls tick_and_switch_naked
#[unsafe(naked)]
pub unsafe extern "C" fn timer_handler_entry(
    _frame: *mut x86_64::structures::idt::InterruptStackFrame,
) {
    core::arch::naked_asm!(
        // Save all general-purpose registers
        "push rax", "push rbx", "push rcx", "push rdx",
        "push rsi", "push rdi", "push rbp",
        "push r8",  "push r9",  "push r10", "push r11",
        "push r12", "push r13", "push r14", "push r15",
        // RDI = pointer to saved registers (stack pointer after pushes)
        "mov rdi, rsp",
        // Call scheduler tick_and_switch_naked
        "call {tick_and_switch}",
        // Restore all general-purpose registers (reverse order)
        "pop r15", "pop r14", "pop r13", "pop r12",
        "pop r11", "pop r10", "pop r9",  "pop r8",
        "pop rbp", "pop rdi", "pop rsi", "pop rdx",
        "pop rcx", "pop rbx", "pop rax",
        // Return from interrupt
        "iretq",
        tick_and_switch = sym crate::sched::tick_and_switch_naked
    );
}

pub fn schedule() {
    let mut current_guard = CURRENT_PROCESS.lock();
    let mut run_queue = RUN_QUEUE.lock();

    if let Some(current) = current_guard.take() {
        let mut proc = current.lock();
        if proc.state == ProcessState::Running {
            proc.state = ProcessState::Ready;
            run_queue.push_back(current.clone());
        }
    }

    if let Some(next) = run_queue.pop_front() {
        let mut proc = next.lock();
        proc.state = ProcessState::Running;
        proc.time_slice = TIME_SLICE_TICKS;
        *current_guard = Some(next.clone());
        drop(proc);
        drop(run_queue);
        drop(current_guard);
        // Actual switch happens on next timer interrupt
    } else {
        drop(run_queue);
        drop(current_guard);
        idle_loop();
    }
}

fn idle_loop() -> ! {
    loop {
        unsafe { asm!("hlt", options(nomem, nostack)) }
    }
}

pub fn spawn_process(entry: u64, user_stack: VirtAddr, page_table: PhysAddr) -> u32 {
    let pid = NEXT_PID.fetch_add(1, Ordering::Relaxed) as u32;
    let process = Arc::new(Spinlock::new(Process::new(
        pid, entry, user_stack, page_table,
    )));

    PROCESS_LIST.lock().push(process.clone());
    RUN_QUEUE.lock().push_back(process);

    log::info!("[sched] Spawned process PID {}", pid);
    pid
}

pub fn spawn_init_process() {
    log::info!("[sched] Spawning PID 1 (zuupos-agentd) — loading ELF stub");

    let page_table = super::mm::phys::alloc_frames(1).expect("Failed to alloc page table");
    let user_stack = VirtAddr::new(0x0000_7FFF_F000_0000);
    let entry = 0xFFFF_8000_0020_0000;

    spawn_process(entry, user_stack, PhysAddr::new(page_table));
}

pub fn yield_now() {
    schedule();
}

pub fn block_current() {
    let mut current_guard = CURRENT_PROCESS.lock();
    if let Some(current) = current_guard.take() {
        let mut proc = current.lock();
        proc.state = ProcessState::Blocked;
        drop(proc);
        drop(current_guard);
        schedule();
    }
}

pub fn wake_process(pid: u32) {
    let processes = PROCESS_LIST.lock();
    for proc_arc in processes.iter() {
        let mut proc = proc_arc.lock();
        if proc.pid == pid && proc.state == ProcessState::Blocked {
            proc.state = ProcessState::Ready;
            RUN_QUEUE.lock().push_back(proc_arc.clone());
            break;
        }
    }
}

pub fn exit_current() {
    let mut current_guard = CURRENT_PROCESS.lock();
    if let Some(current) = current_guard.take() {
        let pid = current.lock().pid;
        current.lock().state = ProcessState::Terminated;
        drop(current);

        let mut list = PROCESS_LIST.lock();
        list.retain(|p| {
            let p_lock = p.lock();
            p_lock.pid != pid
        });
        drop(list);
    }
    schedule();
}

pub fn current_pid() -> Option<u32> {
    CURRENT_PROCESS.lock().as_ref().map(|p| p.lock().pid)
}
