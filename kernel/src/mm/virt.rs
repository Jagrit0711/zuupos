//! Virtual memory manager: 4-level paging on x86-64.

use super::super::arch::x86_64::boot_info::BootInfo;

pub fn init(_boot_info: &BootInfo) {
    // Phase 1.2: Set up a higher-half kernel mapping.
    // Kernel lives at 0xFFFF_8000_0000_0000 and above.
    // Identity-map the lower 4 GiB for I/O + framebuffer.
    log::debug!("[mm::virt] Virtual memory init (Phase 1.2 stub)");
}

/// Map a physical region into virtual space.
pub fn map(phys: u64, virt: u64, size: u64, flags: u64) {
    let _ = (phys, virt, size, flags);
    log::warn!("[mm::virt] map() stub");
}
