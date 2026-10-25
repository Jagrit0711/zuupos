//! Minimal userspace program for ZuuposOS
//! Uses raw syscalls to interact with kernel

#![no_std]
#![no_main]

use core::panic::PanicInfo;

const SYS_WRITE: u64 = 3;
const SYS_YIELD: u64 = 1;
const SYS_GETPID: u64 = 2;
const SYS_EXIT: u64 = 0;
const SYS_FB_WRITE: u64 = 8;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Write to serial console
    write(1, b"Hello from userspace!\n");
    
    // Get PID
    let pid = syscall(SYS_GETPID, 0, 0, 0, 0, 0, 0);
    write(1, b"PID: ");
    write_dec(pid);
    write(1, b"\n");
    
    // Draw a pixel on framebuffer
    syscall(SYS_FB_WRITE, 100, 100, 0x00FF00FF, 0, 0, 0);
    
    // Yield a few times
    for _ in 0..3 {
        syscall(SYS_YIELD, 0, 0, 0, 0, 0, 0);
    }
    
    write(1, b"Userspace exiting\n");
    syscall(SYS_EXIT, 0, 0, 0, 0, 0, 0);
    loop {}
}

fn syscall(num: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> u64 {
    let result: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") num,
            in("rdi") a0,
            in("rsi") a1,
            in("rdx") a2,
            in("r10") a3,
            in("r8") a4,
            in("r9") a5,
            lateout("rax") result,
            options(nostack, preserves_flags)
        );
    }
    result
}

fn write(fd: u64, buf: &[u8]) {
    syscall(SYS_WRITE, fd, buf.as_ptr() as u64, buf.len() as u64, 0, 0, 0);
}

fn write_dec(mut n: u64) {
    if n == 0 {
        write(1, b"0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    buf[..i].reverse();
    write(1, &buf[..i]);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    write(1, b"PANIC: panic occurred\n");
    syscall(SYS_EXIT, 1, 0, 0, 0, 0, 0);
    loop {}
}