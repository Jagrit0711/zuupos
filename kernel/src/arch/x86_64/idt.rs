//! Interrupt Descriptor Table (IDT) for x86-64.

use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spinning_top::Spinlock;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
pub const SYSCALL_VECTOR: u8 = 0x80;

pub static PICS: Spinlock<ChainedPics> =
    Spinlock::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.general_protection_fault.set_handler_fn(gpf_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        // Timer IRQ (IRQ0 → vector 0x20 / 32)
        idt[PIC_1_OFFSET].set_handler_fn(timer_handler);
        // Keyboard IRQ (IRQ1 → vector 0x21 / 33)
        idt[PIC_1_OFFSET + 1].set_handler_fn(keyboard_handler);
        // Syscall (int 0x80) - allowed from Ring 3
        idt[SYSCALL_VECTOR]
            .set_handler_fn(syscall_handler)
            .set_privilege_level(x86_64::PrivilegeLevel::Ring3);
        idt
    };
}

pub fn init() {
    IDT.load();
    unsafe { PICS.lock().initialize() };
}

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    log::debug!("EXCEPTION: BREAKPOINT\n{:#?}", frame);
}

extern "x86-interrupt" fn gpf_handler(frame: InterruptStackFrame, error_code: u64) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT (code={:#x})\n{:#?}",
        error_code, frame
    );
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", frame);
}

extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    panic!(
        "EXCEPTION: PAGE FAULT\n  Accessed address: {:?}\n  Error code: {:?}\n{:#?}",
        Cr2::read(),
        error_code,
        frame,
    );
}

// Timer handler wrapper that calls the naked function in sched
extern "x86-interrupt" fn timer_handler(mut frame: InterruptStackFrame) {
    unsafe {
        crate::sched::timer_handler_entry(&mut frame as *mut InterruptStackFrame);
    }
}

extern "x86-interrupt" fn keyboard_handler(_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    let mut port: Port<u8> = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::drivers::keyboard::handle_scancode(scancode);
    unsafe { PICS.lock().notify_end_of_interrupt(PIC_1_OFFSET + 1) };
}

// Syscall handler (int 0x80) - naked to preserve all registers
#[unsafe(naked)]
extern "x86-interrupt" fn syscall_handler(_frame: InterruptStackFrame) {
    unsafe {
        core::arch::naked_asm!(
            // Save all general-purpose registers
            "push rax", "push rbx", "push rcx", "push rdx",
            "push rsi", "push rdi", "push rbp",
            "push r8",  "push r9",  "push r10", "push r11",
            "push r12", "push r13", "push r14", "push r15",
            // Syscall number in RAX, args in RDI, RSI, RDX, R10, R8, R9
            // Call handle_syscall with registers on stack
            "mov rdi, rsp",
            "call {handle_syscall_inline}",
            // Restore registers
            "pop r15", "pop r14", "pop r13", "pop r12",
            "pop r11", "pop r10", "pop r9",  "pop r8",
            "pop rbp", "pop rdi", "pop rsi", "pop rdx",
            "pop rcx", "pop rbx", "pop rax",
            // Return from interrupt
            "iretq",
            handle_syscall_inline = sym syscall_handler_inline
        );
    }
}

// Inline syscall handler - avoids module visibility issues
#[repr(C)]
struct SyscallRegs {
    rax: u64, rbx: u64, rcx: u64, rdx: u64,
    rsi: u64, rdi: u64, rbp: u64, _rsp: u64,
    r8: u64, r9: u64, r10: u64, r11: u64,
    r12: u64, r13: u64, r14: u64, r15: u64,
}

#[no_mangle]
pub extern "C" fn syscall_handler_inline(regs: &mut SyscallRegs) {
    let syscall_num = regs.rax;
    let arg0 = regs.rdi;
    let arg1 = regs.rsi;
    let arg2 = regs.rdx;
    let arg3 = regs.r10;
    let arg4 = regs.r8;
    let _arg5 = regs.r9;

    let result = match syscall_num {
        0 => { // SYS_EXIT
            crate::sched::exit_current();
            0 // Never returns
        }
        1 => { // SYS_YIELD
            crate::sched::yield_now();
            0
        }
        2 => { // SYS_GETPID
            crate::sched::current_pid().unwrap_or(0) as u64
        }
        3 => { // SYS_WRITE
            syscall_write(arg0, arg1, arg2)
        }
        4 => { // SYS_MAP_MEMORY
            crate::mm::virt::map(arg0, arg1, arg2, arg3);
            arg1
        }
        5 => { // SYS_ALLOC_MEMORY
            crate::mm::phys::alloc_frames(arg0 as usize).unwrap_or(0)
        }
        6 => { // SYS_FREE_MEMORY
            crate::mm::phys::free_frames(arg0, arg1 as usize);
            0
        }
        7 => { // SYS_SPAWN
            crate::sched::spawn_process(arg0, x86_64::VirtAddr::new(arg1), x86_64::PhysAddr::new(arg2)) as u64
        }
        8 => { // SYS_FB_WRITE
            crate::drivers::framebuffer::put_pixel(arg0 as u32, arg1 as u32, arg2 as u32);
            0
        }
        9 => { // SYS_IPC_SEND
            syscall_ipc_send(arg0, arg1, arg2)
        }
        10 => { // SYS_IPC_RECV
            syscall_ipc_recv(arg0, arg1, arg2)
        }
        _ => {
            log::warn!("[syscall] Unknown syscall: {}", syscall_num);
            u64::MAX
        }
    };
    regs.rax = result;
}

fn syscall_write(fd: u64, buf: u64, len: u64) -> u64 {
    if fd != 1 && fd != 2 {
        return 0;
    }
    use x86_64::instructions::port::Port;
    let mut serial = Port::<u8>::new(0x3F8);
    let slice = unsafe { core::slice::from_raw_parts(buf as *const u8, len as usize) };
    for &byte in slice {
        unsafe { serial.write(byte); }
    }
    len
}

fn syscall_ipc_send(channel_id: u64, msg_ptr: u64, msg_len: u64) -> u64 {
    // TODO: Implement IPC send
    // For now, return success
    0
}

fn syscall_ipc_recv(channel_id: u64, buf_ptr: u64, buf_len: u64) -> u64 {
    // TODO: Implement IPC recv
    0
}
