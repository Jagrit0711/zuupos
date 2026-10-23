//! System call interface for userspace processes.

use crate::mm::virt::map;
use crate::mm::phys::{alloc_frames, free_frames};
use crate::sched::{spawn_process, current_pid, yield_now, exit_current};
use crate::drivers::framebuffer;
use x86_64::PhysAddr;

pub const SYS_EXIT: u64 = 0;
pub const SYS_YIELD: u64 = 1;
pub const SYS_GETPID: u64 = 2;
pub const SYS_WRITE: u64 = 3;
pub const SYS_MAP_MEMORY: u64 = 4;
pub const SYS_ALLOC_MEMORY: u64 = 5;
pub const SYS_FREE_MEMORY: u64 = 6;
pub const SYS_SPAWN: u64 = 7;
pub const SYS_FB_WRITE: u64 = 8;
pub const SYS_IPC_SEND: u64 = 9;
pub const SYS_IPC_RECV: u64 = 10;

#[repr(C)]
pub struct SyscallRegs {
    pub rax: u64, pub rbx: u64, pub rcx: u64, pub rdx: u64,
    pub rsi: u64, pub rdi: u64, pub rbp: u64, pub _rsp: u64,
    pub r8: u64, pub r9: u64, pub r10: u64, pub r11: u64,
    pub r12: u64, pub r13: u64, pub r14: u64, pub r15: u64,
}

#[no_mangle]
pub extern "C" fn handle_syscall_naked(regs: &mut SyscallRegs) {
    let syscall_num = regs.rax;
    let arg0 = regs.rdi;
    let arg1 = regs.rsi;
    let arg2 = regs.rdx;
    let arg3 = regs.r10;
    let arg4 = regs.r8;
    let _arg5 = regs.r9;

    let result = handle(syscall_num, arg0, arg1, arg2, arg3, arg4, _arg5);
    regs.rax = result;
}

pub fn handle(
    num: u64,
    arg0: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    _arg5: u64,
) -> u64 {
    match num {
        SYS_EXIT => {
            exit_current();
            0 // Never returns
        }
        SYS_YIELD => {
            yield_now();
            0
        }
        SYS_GETPID => {
            current_pid().unwrap_or(0) as u64
        }
        SYS_WRITE => {
            write_to_console(arg0, arg1, arg2)
        }
        SYS_MAP_MEMORY => {
            map(arg0, arg1, arg2, arg3);
            arg1
        }
        SYS_ALLOC_MEMORY => {
            alloc_frames(arg0 as usize).unwrap_or(0)
        }
        SYS_FREE_MEMORY => {
            free_frames(arg0, arg1 as usize);
            0
        }
        SYS_SPAWN => {
            spawn_process(arg0, x86_64::VirtAddr::new(arg1), PhysAddr::new(arg2)) as u64
        }
        SYS_FB_WRITE => {
            framebuffer::put_pixel(arg0 as u32, arg1 as u32, arg2 as u32);
            0
        }
        SYS_IPC_SEND => 0,
        SYS_IPC_RECV => 0,
        _ => {
            log::warn!("[syscall] Unknown syscall: {}", num);
            u64::MAX
        }
    }
}

fn write_to_console(fd: u64, buf: u64, len: u64) -> u64 {
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