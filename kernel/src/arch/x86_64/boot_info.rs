//! Boot information structure shared between bootloader and kernel.
//! MUST be kept in sync with `bootloader/src/main.rs::BootInfo`.

#[repr(C)]
pub struct BootInfo {
    pub memory_map_addr: u64,
    pub memory_map_len: usize,
    pub memory_map_desc_size: usize,
    pub fb_addr: u64,
    pub fb_width: u32,
    pub fb_height: u32,
    pub fb_stride: u32,
    pub fb_pixel_format: u32,
    pub kernel_phys_start: u64,
    pub kernel_phys_end: u64,
    pub initramfs_phys_start: u64,
    pub initramfs_size: u64,
    pub rsdp_addr: u64,
    pub pml4_addr: u64,
    pub physical_memory_offset: u64,
}
