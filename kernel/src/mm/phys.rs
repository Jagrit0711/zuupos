//! Physical frame allocator using a buddy system.
//! Initialized from the UEFI memory map provided by the bootloader.

use super::super::arch::x86_64::boot_info::BootInfo;
use spinning_top::Spinlock;

/// Total number of physical frames (populated at init).
static TOTAL_FRAMES: Spinlock<u64> = Spinlock::new(0);

pub fn init(boot_info: &BootInfo) {
    // Phase 1.2: Walk the UEFI memory map and build a buddy allocator.
    // Each UEFI MemoryDescriptor describes a contiguous run of pages.
    // We mark Conventional + BootServicesCode/Data + LoaderCode/Data as free
    // after exit_boot_services() was called.
    let _mmap_addr = boot_info.memory_map_addr;
    log::debug!(
        "[mm::phys] UEFI memory map @ {:#x}, {} entries (Phase 1.2 stub)",
        boot_info.memory_map_addr,
        boot_info.memory_map_len / boot_info.memory_map_desc_size,
    );
}

/// Allocate a contiguous run of `n` physical page frames (4KiB each).
/// Returns the physical address of the first frame.
pub fn alloc_frames(n: usize) -> Option<u64> {
    // TODO Phase 1.2: Buddy allocator implementation
    log::warn!("[mm::phys] alloc_frames({}) — stub, returning None", n);
    None
}

pub fn free_frames(base: u64, n: usize) {
    // TODO Phase 1.2
    let _ = (base, n);
}
