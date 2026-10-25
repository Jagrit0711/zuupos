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
mod initramfs;
mod ipc;
mod loader;
mod mm;
mod sched;
mod syscall;

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
    info!("║   ZuuposOS Kernel v0.5.2 booting...  ║");
    info!("╚══════════════════════════════════════╝");

    // 2. Initialize x86-64 CPU structures: GDT, IDT, TSS
    arch::x86_64::init();
    info!("[arch] GDT/IDT/TSS initialized");

    // 3. Initialize heap allocator
    mm::heap::init();
    info!("[mm] Heap allocator online");

    // 4. Initialize physical memory manager from UEFI memory map
    mm::phys::init(boot_info);
    info!("[mm] Physical memory manager initialized");

    // 5. Initialize virtual memory manager
    mm::virt::init(boot_info);
    info!("[mm] Virtual memory initialized");

    // 6. Initialize framebuffer display
    drivers::framebuffer::init(boot_info);
    info!("[drv] Framebuffer display initialized");

    // 7. Initialize PCI subsystem
    drivers::pci::init();
    info!("[pci] PCI enumeration complete");

    // 8. Initialize NVMe driver
    drivers::nvme::init();
    info!("[nvme] NVMe driver initialized");

    // 9. Initialize capability-based IPC
    ipc::init();
    info!("[ipc] Capability IPC subsystem initialized");

    // 10. Enable hardware interrupts
    x86_64::instructions::interrupts::enable();
    info!("[irq] Interrupts enabled");

    // 11. Initialize scheduler
    sched::init();
    info!("[sched] Scheduler initialized");

    // 12. Spawn initial process from initramfs (PID 1)
    info!("[kernel] Launching initial process from initramfs...");
    sched::spawn_init_from_initramfs(boot_info);

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
