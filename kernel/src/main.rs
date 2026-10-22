//! ZuuposOS Kernel Entry Point
//!
//! This is called by the bootloader after it exits UEFI boot services,
//! sets up page tables, and jumps here with a `BootInfo` pointer.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod arch;
mod drivers;
mod ipc;
mod mm;
mod sched;

use core::panic::PanicInfo;
use log::info;

/// Kernel entry point. Called with a pointer to `BootInfo` from bootloader.
#[no_mangle]
pub extern "C" fn _start(boot_info: *const arch::x86_64::boot_info::BootInfo) -> ! {
    // SAFETY: Bootloader guarantees this pointer is valid.
    let boot_info = unsafe { &*boot_info };

    // 1. Initialize early UART serial console (for debug output before framebuffer)
    drivers::serial::init();
    log::set_logger(&drivers::serial::SERIAL_LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .expect("Failed to set logger");

    info!("╔══════════════════════════════════════╗");
    info!("║   ZuuposOS Kernel v0.1.0 booting...  ║");
    info!("╚══════════════════════════════════════╝");

    // 2. Initialize x86-64 CPU structures: GDT, IDT, TSS
    arch::x86_64::init();
    info!("[arch] GDT/IDT/TSS initialized");

    // 3. Initialize physical memory manager from UEFI memory map
    mm::phys::init(boot_info);
    info!("[mm] Physical memory manager initialized");

    // 4. Initialize virtual memory manager
    mm::virt::init(boot_info);
    info!("[mm] Virtual memory initialized");

    // 5. Initialize heap allocator
    mm::heap::init();
    info!("[mm] Heap allocator online");

    // 6. Initialize framebuffer display
    drivers::framebuffer::init(boot_info);
    info!("[drv] Framebuffer display initialized");

    // 7. Initialize ACPI (for timer, CPU topology)
    arch::x86_64::acpi::init(boot_info);
    info!("[acpi] ACPI initialized");

    // 8. Enable hardware interrupts
    x86_64::instructions::interrupts::enable();
    info!("[irq] Interrupts enabled");

    // 9. Initialize scheduler
    sched::init();
    info!("[sched] Scheduler initialized");

    // 10. Spawn agentd as PID 1 — the AI agent daemon
    //     In Phase 1.5, this will load and run the agentd ELF.
    //     For now, we enter a stub agent loop.
    info!("[kernel] Handing control to zuupos-agentd (PID 1)...");
    sched::spawn_init_process();

    // Kernel idle loop — never returns
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("KERNEL PANIC: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}
