//! ZuuposOS UEFI Bootloader
//!
//! Phase 1.1 — Boot entry point.
//! Responsibility: initialize UEFI services, set up a framebuffer,
//! load the kernel ELF from disk, set up page tables, and jump to
//! the kernel entry point.

#![no_std]
#![no_main]

extern crate alloc;

use log::info;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::table::boot::{MemoryType, OpenProtocolAttributes, OpenProtocolParams};

/// Boot information passed to the kernel.
/// Must be kept in sync with `kernel/src/arch/x86_64/boot_info.rs`.
#[repr(C)]
pub struct BootInfo {
    /// Physical address of the UEFI memory map.
    pub memory_map_addr: u64,
    pub memory_map_len: usize,
    pub memory_map_desc_size: usize,
    /// Framebuffer info
    pub fb_addr: u64,
    pub fb_width: u32,
    pub fb_height: u32,
    pub fb_stride: u32,
    pub fb_pixel_format: u32, // 0=RGB, 1=BGR
    /// Kernel ELF physical load address
    pub kernel_phys_start: u64,
    pub kernel_phys_end: u64,
    /// RSDP (ACPI root) pointer
    pub rsdp_addr: u64,
}

#[entry]
fn efi_main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    // Initialize UEFI services (allocator, logger, etc.)
    uefi_services::init(&mut st).unwrap();

    let bt = st.boot_services();

    info!("╔══════════════════════════════════════╗");
    info!("║          Z U U P O S   O S           ║");
    info!("║     AI-Native Agentic OS — Boot      ║");
    info!("╚══════════════════════════════════════╝");
    info!("Bootloader Phase 1.1 initializing...");

    // --- Acquire framebuffer via GOP ---
    let fb_info = acquire_framebuffer(bt, image);
    info!(
        "Framebuffer: {}x{} @ {:#x}, stride={}",
        fb_info.0, fb_info.1, fb_info.2, fb_info.3
    );

    // --- Draw boot splash ---
    draw_boot_splash(fb_info.2, fb_info.0, fb_info.1, fb_info.3);

    // TODO Phase 1.1: Load kernel ELF from ESP partition
    // TODO Phase 1.1: Parse ELF, map segments into virtual memory
    // TODO Phase 1.1: Build final page tables (4-level paging)
    // TODO Phase 1.1: Exit boot services, jump to kernel _start

    info!("ZuuposOS bootloader initialized. Kernel load: TODO.");
    info!("Halting in QEMU for now — Phase 1.1 stub.");

    loop {
        // Spin; real impl will jump to kernel
        unsafe { core::arch::asm!("hlt") }
    }
}

/// Returns (width, height, base_addr, stride, pixel_format)
fn acquire_framebuffer(bt: &BootServices, image: Handle) -> (u32, u32, u64, u32, PixelFormat) {
    let handle = bt
        .get_handle_for_protocol::<GraphicsOutput>()
        .expect("No GOP handle");

    let mut gop = unsafe {
        bt.open_protocol::<GraphicsOutput>(
            OpenProtocolParams {
                handle,
                agent: image,
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
        .expect("Failed to open GOP")
    };

    let mode = gop.current_mode_info();
    let (width, height) = mode.resolution();
    let stride = mode.stride() as u32;
    let pf = mode.pixel_format();
    let fb_addr = gop.frame_buffer().as_mut_ptr() as u64;

    (width as u32, height as u32, fb_addr, stride, pf)
}

/// Draw a minimal boot splash to show the framebuffer works.
fn draw_boot_splash(fb_addr: u64, width: u32, height: u32, stride: u32) {
    let fb = fb_addr as *mut u32;
    // Deep dark background: #0a0a0f
    let bg: u32 = 0x000a_0a0f;
    // Accent purple: #7c3aed
    let accent: u32 = 0x007c_3aed;

    unsafe {
        for y in 0..height {
            for x in 0..width {
                let pixel = if y < 4 || y > height - 5 || x < 4 || x > width - 5 {
                    accent
                } else {
                    bg
                };
                fb.add((y * stride + x) as usize).write_volatile(pixel);
            }
        }
    }
}
