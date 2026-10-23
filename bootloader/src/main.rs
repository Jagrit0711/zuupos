//! ZuuposOS UEFI Bootloader
//!
//! Phase 1.1 — Boot entry point.
//! Responsibility: initialize UEFI services, set up a framebuffer,
//! load the kernel ELF from disk, set up page tables, and jump to
//! the kernel entry point.

#![no_std]
#![no_main]

extern crate alloc;

#[global_allocator]
static ALLOCATOR: uefi::allocator::Allocator = uefi::allocator::Allocator;

use alloc::vec::Vec;
use core::mem;
use log::info;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::proto::media::file::{File, FileAttribute, FileMode, FileInfo, FileType};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::mem::memory_map::{MemoryMap, MemoryType};
use uefi::system::with_config_table;
use uefi::table::cfg::{ACPI2_GUID, ACPI_GUID};
use uefi::data_types::PhysicalAddress;
use uefi::boot::{AllocateType, MemoryDescriptor};
use uefi::cstr16;

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
    /// Page table (PML4) physical address
    pub pml4_addr: u64,
    /// Physical memory offset for higher-half kernel
    pub physical_memory_offset: u64,
}

#[entry]
fn main() -> Status {
    let _ = uefi::helpers::init();

    info!("╔══════════════════════════════════════╗");
    info!("║          Z U U P O S   O S           ║");
    info!("║     AI-Native Agentic OS — Boot      ║");
    info!("╚══════════════════════════════════════╝");
    info!("Bootloader Phase 1.1 initializing...");

    // --- Acquire framebuffer via GOP ---
    let (width, height, fb_addr, stride, pixel_format) = acquire_framebuffer().expect("Failed to acquire framebuffer");
    info!("Framebuffer: {}x{} @ {:#x}, stride={}", width, height, fb_addr, stride);
    draw_boot_splash(fb_addr, width, height, stride);

    // --- Get memory map ---
    let (memory_map_addr, memory_map_len, memory_map_desc_size) = get_memory_map().expect("Failed to get memory map");
    info!("Memory map: {} entries @ {:#x}", memory_map_len / memory_map_desc_size, memory_map_addr);

    // --- Find RSDP (ACPI) ---
    let rsdp_addr = find_rsdp().unwrap_or(0);
    info!("RSDP: {:#x}", rsdp_addr);

    // --- Load kernel ELF ---
    let (kernel_phys_start, kernel_phys_end, kernel_entry) = load_kernel().expect("Failed to load kernel");
    info!("Kernel loaded: {:#x} - {:#x}, entry: {:#x}", kernel_phys_start, kernel_phys_end, kernel_entry);

    // --- Set up page tables (higher-half kernel) ---
    let (pml4_addr, physical_memory_offset) = setup_page_tables(kernel_phys_start, kernel_phys_end, fb_addr, width, height, stride).expect("Failed to set up page tables");
    info!("Page tables: PML4={:#x}, phys_offset={:#x}", pml4_addr, physical_memory_offset);

    // --- Prepare BootInfo ---
    let boot_info = BootInfo {
        memory_map_addr,
        memory_map_len,
        memory_map_desc_size,
        fb_addr,
        fb_width: width,
        fb_height: height,
        fb_stride: stride,
        fb_pixel_format: pixel_format,
        kernel_phys_start,
        kernel_phys_end,
        rsdp_addr,
        pml4_addr,
        physical_memory_offset,
    };

    // --- Exit boot services ---
    info!("Exiting boot services...");
    let _memory_map = unsafe { uefi::boot::exit_boot_services(MemoryType::LOADER_DATA) };

    // --- Jump to kernel ---
    let boot_info_ptr = &boot_info as *const BootInfo;
    info!("Jumping to kernel at {:#x}...", kernel_entry);
    unsafe {
        core::arch::asm!(
            "jmp rax",
            in("rax") kernel_entry,
            in("rdi") boot_info_ptr,
            options(noreturn)
        );
    }
}

/// Returns (width, height, base_addr, stride, pixel_format)
fn acquire_framebuffer() -> Option<(u32, u32, u64, u32, u32)> {
    let handle = uefi::boot::get_handle_for_protocol::<GraphicsOutput>().ok()?;
    let mut gop = uefi::boot::open_protocol_exclusive::<GraphicsOutput>(handle).ok()?;

    let mode = gop.current_mode_info();
    let (width, height) = mode.resolution();
    let stride = mode.stride() as u32;
    let fb_addr = gop.frame_buffer().as_mut_ptr() as u64;
    let pixel_format = match mode.pixel_format() {
        PixelFormat::Rgb => 0,
        PixelFormat::Bgr => 1,
        _ => 0,
    } as u32;

    Some((width as u32, height as u32, fb_addr, stride, pixel_format))
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

fn get_memory_map() -> Option<(u64, usize, usize)> {
    let mmap = uefi::boot::memory_map(MemoryType::LOADER_DATA).ok()?;
    let meta = mmap.meta();
    let desc_size = meta.desc_size;
    let len = mmap.len();
    // Copy descriptors to a new buffer we own
    let mut buffer = Vec::with_capacity(len);
    for desc in mmap.entries() {
        // MemoryDescriptor fields are: ty, phys_start, page_count
        buffer.push(*desc);
    }
    let addr = buffer.as_ptr() as u64;
    mem::forget(buffer);
    Some((addr, len, desc_size))
}

fn find_rsdp() -> Option<u64> {
    with_config_table(|entries| {
        for entry in entries {
            if entry.guid == ACPI2_GUID || entry.guid == ACPI_GUID {
                return Some(entry.address as u64);
            }
        }
        None
    })
}

fn load_kernel() -> Option<(u64, u64, u64)> {
    let handle = uefi::boot::get_handle_for_protocol::<SimpleFileSystem>().ok()?;
    let mut fs = uefi::boot::open_protocol_exclusive::<SimpleFileSystem>(handle).ok()?;

    let mut root_dir = fs.open_volume().ok()?;
    let path = cstr16!("\\EFI\\ZUUPOS\\KERNEL.ELF");
    let mut kernel_file = root_dir.open(path, FileMode::Read, FileAttribute::empty()).ok()?;

    if let FileType::Regular(mut file) = kernel_file.into_type().ok()? {
        let mut info_buffer = [0u8; 256];
        let file_size = file.get_info::<FileInfo>(&mut info_buffer).ok()?.file_size() as usize;
        let mut buffer = Vec::with_capacity(file_size);
        file.read(&mut buffer).ok()?;

        let kernel_start = find_kernel_load_address(&buffer)?;
        let kernel_end = kernel_start + file_size as u64;

        // Copy kernel to physical memory
        unsafe {
            core::ptr::copy_nonoverlapping(buffer.as_ptr(), kernel_start as *mut u8, file_size);
        }

        let entry = parse_elf_entry(&buffer, kernel_start)?;
        Some((kernel_start, kernel_end, entry))
    } else {
        None
    }
}

fn find_kernel_load_address(_buffer: &[u8]) -> Option<u64> {
    // Simple ELF parsing to find the load address
    // For now, use a fixed address: 2 MiB (after first 2 MiB reserved for bootloader)
    Some(0x200000)
}

fn parse_elf_entry(buffer: &[u8], load_addr: u64) -> Option<u64> {
    if buffer.len() < 64 {
        return None;
    }
    // Check ELF magic
    if &buffer[0..4] != b"\x7fELF" {
        return None;
    }
    // 64-bit, little endian
    if buffer[4] != 2 || buffer[5] != 1 {
        return None;
    }
    // Entry point at offset 24 (8 bytes)
    let entry = u64::from_le_bytes([
        buffer[24], buffer[25], buffer[26], buffer[27],
        buffer[28], buffer[29], buffer[30], buffer[31],
    ]);
    Some(load_addr + entry)
}

fn setup_page_tables(
    kernel_start: u64,
    kernel_end: u64,
    fb_addr: u64,
    fb_width: u32,
    fb_height: u32,
    fb_stride: u32,
) -> Option<(u64, u64)> {
    const PAGE_SIZE: u64 = 4096;
    const KERNEL_BASE: u64 = 0xFFFF_8000_0000_0000;
    const IDENTITY_MAP_LIMIT: u64 = 4 * 1024 * 1024 * 1024;

    // Allocate page tables (PML4, PDPT, PD, PT)
    let pml4_addr = alloc_page()?;
    let pdpt_addr = alloc_page()?;
    let pd_addr = alloc_page()?;
    let pt_addr = alloc_page()?;

    // Zero the pages
    unsafe {
        core::ptr::write_bytes(pml4_addr as *mut u8, 0, PAGE_SIZE as usize);
        core::ptr::write_bytes(pdpt_addr as *mut u8, 0, PAGE_SIZE as usize);
        core::ptr::write_bytes(pd_addr as *mut u8, 0, PAGE_SIZE as usize);
        core::ptr::write_bytes(pt_addr as *mut u8, 0, PAGE_SIZE as usize);
    }

    // Map PML4[511] -> PDPT (higher-half kernel)
    // Map PML4[0] -> PDPT (identity map low memory)
    let pml4 = pml4_addr as *mut u64;
    let pdpt = pdpt_addr as *mut u64;
    let pd = pd_addr as *mut u64;
    let pt = pt_addr as *mut u64;

    // Higher-half kernel: PML4[511] -> PDPT
    unsafe { *pml4.add(511) = pdpt_addr | 0x3 }; // Present | Writable

    // Identity map: PML4[0] -> PDPT
    unsafe { *pml4.add(0) = pdpt_addr | 0x3 };

    // PDPT[0] -> PD (for both mappings, sharing)
    unsafe { *pdpt.add(0) = pd_addr | 0x3 };

    // PD[0] -> PT (first 2 MiB)
    unsafe { *pd.add(0) = pt_addr | 0x3 };

    // Map first 2 MiB identity (4 KiB pages)
    for i in 0..512usize {
        let phys = i as u64 * PAGE_SIZE;
        unsafe { *pt.add(i) = phys | 0x3 };
    }

    // Map kernel in higher-half (2 MiB pages)
    let kernel_pages = (kernel_end - kernel_start + PAGE_SIZE - 1) / PAGE_SIZE;
    let pd_kernel = alloc_page()?;
    unsafe { core::ptr::write_bytes(pd_kernel as *mut u8, 0, PAGE_SIZE as usize) };
    unsafe { *pdpt.add(1) = pd_kernel | 0x3 }; // PDPT[1] -> kernel PD

    for i in 0..(kernel_pages as usize).min(512) {
        let phys = kernel_start + i as u64 * PAGE_SIZE;
        let virt = KERNEL_BASE + phys;
        let pd_idx = ((virt >> 21) & 0x1FF) as usize;
        let pt_idx = ((virt >> 12) & 0x1FF) as usize;

        if pt_idx == 0 {
            let pt_kernel = alloc_page()?;
            unsafe { core::ptr::write_bytes(pt_kernel as *mut u8, 0, PAGE_SIZE as usize) };
            unsafe { *(pd_kernel as *mut u64).add(pd_idx) = pt_kernel | 0x3 };
        }
        let pt_kernel = unsafe { *(pd_kernel as *mut u64).add(pd_idx) & !0xFFF };
        unsafe { *(pt_kernel as *mut u64).add(pt_idx) = phys | 0x3 };
    }

    // Map framebuffer in higher-half
    let fb_size = fb_stride as u64 * fb_height as u64 * 4;
    let fb_pages = (fb_size + PAGE_SIZE - 1) / PAGE_SIZE;
    for i in 0..fb_pages as usize {
        let phys = fb_addr + (i as u64) * PAGE_SIZE;
        let virt = KERNEL_BASE + phys;
        let pd_idx = ((virt >> 21) & 0x1FF) as usize;
        let pt_idx = ((virt >> 12) & 0x1FF) as usize;

        if pt_idx == 0 {
            let pt_fb = alloc_page()?;
            unsafe { core::ptr::write_bytes(pt_fb as *mut u8, 0, PAGE_SIZE as usize) };
            unsafe { *(pd_kernel as *mut u64).add(pd_idx) = pt_fb | 0x3 };
        }
        let pt_fb = unsafe { *(pd_kernel as *mut u64).add(pd_idx) & !0xFFF };
        unsafe { *(pt_fb as *mut u64).add(pt_idx) = phys | 0x3 | 0x80000000000 }; // NX bit
    }

    // Physical memory offset = KERNEL_BASE (for higher-half direct mapping)
    Some((pml4_addr, KERNEL_BASE))
}

fn alloc_page() -> Option<u64> {
    let mmap = uefi::boot::memory_map(MemoryType::LOADER_DATA).ok()?;

    for desc in mmap.entries() {
        if desc.ty == MemoryType::CONVENTIONAL {
            let pages = desc.page_count;
            if pages > 0 {
                let phys = desc.phys_start;
                // Mark as used by allocating
                let status = uefi::boot::allocate_pages(
                    AllocateType::Address(phys as PhysicalAddress),
                    MemoryType::LOADER_DATA,
                    1,
                );
                if status.is_ok() {
                    return Some(phys);
                }
            }
        }
    }
    None
}