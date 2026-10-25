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
use uefi::boot::{AllocateType, MemoryDescriptor};
use uefi::cstr16;
use uefi::data_types::PhysicalAddress;
use uefi::mem::memory_map::{MemoryMap, MemoryType};
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::system::with_config_table;
use uefi::table::cfg::{ACPI2_GUID, ACPI_GUID};

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
    /// Initramfs physical address and size
    pub initramfs_phys_start: u64,
    pub initramfs_size: u64,
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
    let (width, height, fb_addr, stride, pixel_format) =
        acquire_framebuffer().expect("Failed to acquire framebuffer");
    info!(
        "Framebuffer: {}x{} @ {:#x}, stride={}",
        width, height, fb_addr, stride
    );
    draw_boot_splash(fb_addr, width, height, stride);

    // --- Get memory map ---
    let (memory_map_addr, memory_map_len, memory_map_desc_size) =
        get_memory_map().expect("Failed to get memory map");
    info!(
        "Memory map: {} entries @ {:#x}",
        memory_map_len / memory_map_desc_size,
        memory_map_addr
    );

    // --- Find RSDP (ACPI) ---
    let rsdp_addr = find_rsdp().unwrap_or(0);
    info!("RSDP: {:#x}", rsdp_addr);

    // --- Load kernel ELF ---
    let (kernel_phys_start, kernel_phys_end, kernel_entry) =
        load_kernel().expect("Failed to load kernel");
    info!(
        "Kernel loaded: {:#x} - {:#x}, entry: {:#x}",
        kernel_phys_start, kernel_phys_end, kernel_entry
    );

    // --- Load initramfs ---
    let (initramfs_phys_start, initramfs_size) =
        load_initramfs().unwrap_or((0, 0));
    if initramfs_size > 0 {
        info!(
            "Initramfs loaded: {:#x} ({} bytes)",
            initramfs_phys_start, initramfs_size
        );
    } else {
        info!("No initramfs found");
    }

    // --- Set up page tables (higher-half kernel) ---
    let (pml4_addr, physical_memory_offset) = setup_page_tables(
        kernel_phys_start,
        kernel_phys_end,
        fb_addr,
        width,
        height,
        stride,
    )
    .expect("Failed to set up page tables");
    info!(
        "Page tables: PML4={:#x}, phys_offset={:#x}",
        pml4_addr, physical_memory_offset
    );

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
        initramfs_phys_start,
        initramfs_size,
        rsdp_addr,
        pml4_addr,
        physical_memory_offset,
    };

    // --- Exit boot services and jump to kernel ---
    info!("Jumping to kernel at {:#x}...", kernel_entry);
    info!("Exiting boot services...");
    let _memory_map = unsafe { uefi::boot::exit_boot_services(MemoryType::LOADER_DATA) };

    let boot_info_ptr = &boot_info as *const BootInfo;
    unsafe {
        // Switch to our PML4 page tables
        core::arch::asm!("mov cr3, {0}", in(reg) pml4_addr);
        // Jump to kernel entry point
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MemoryRegion {
    pub phys_start: u64,
    pub page_count: u64,
}

fn get_memory_map() -> Option<(u64, usize, usize)> {
    let mmap = uefi::boot::memory_map(MemoryType::LOADER_DATA).ok()?;
    let mut regions = Vec::new();
    for desc in mmap.entries() {
        if desc.ty == MemoryType::CONVENTIONAL {
            regions.push(MemoryRegion {
                phys_start: desc.phys_start,
                page_count: desc.page_count,
            });
        }
    }
    let count = regions.len();
    let desc_size = core::mem::size_of::<MemoryRegion>();
    let addr = regions.as_ptr() as u64;
    mem::forget(regions);
    Some((addr, count, desc_size))
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
    let kernel_file = root_dir
        .open(path, FileMode::Read, FileAttribute::empty())
        .ok()?;

    if let FileType::Regular(mut file) = kernel_file.into_type().ok()? {
        let mut info_buffer = [0u8; 256];
        let file_size = file
            .get_info::<FileInfo>(&mut info_buffer)
            .ok()?
            .file_size() as usize;
        let mut buffer = alloc::vec![0u8; file_size];
        file.read(&mut buffer).ok()?;

        if buffer.len() < 64 || &buffer[0..4] != b"\x7fELF" {
            return None;
        }

        let e_entry = u64::from_le_bytes(buffer[24..32].try_into().ok()?);
        let e_phoff = u64::from_le_bytes(buffer[32..40].try_into().ok()?) as usize;
        let e_phentsize = u16::from_le_bytes(buffer[54..56].try_into().ok()?) as usize;
        let e_phnum = u16::from_le_bytes(buffer[56..58].try_into().ok()?) as usize;

        let kernel_start: u64 = 0x200000;
        let mut kernel_end: u64 = kernel_start;

        // Parse program headers and load PT_LOAD segments
        for i in 0..e_phnum {
            let ph_offset = e_phoff + i * e_phentsize;
            if ph_offset + 56 > buffer.len() {
                break;
            }
            let ph = &buffer[ph_offset..ph_offset + 56];
            let p_type = u32::from_le_bytes(ph[0..4].try_into().ok()?);
            if p_type == 1 {
                // PT_LOAD
                let p_offset = u64::from_le_bytes(ph[8..16].try_into().ok()?) as usize;
                let p_vaddr = u64::from_le_bytes(ph[16..24].try_into().ok()?);
                let p_filesz = u64::from_le_bytes(ph[32..40].try_into().ok()?) as usize;
                let p_memsz = u64::from_le_bytes(ph[40..48].try_into().ok()?) as usize;

                let dest_addr = if p_vaddr >= kernel_start {
                    p_vaddr
                } else {
                    kernel_start + p_vaddr
                };
                let dest = dest_addr as *mut u8;
                if p_filesz > 0 && p_offset + p_filesz <= buffer.len() {
                    unsafe {
                        core::ptr::copy_nonoverlapping(buffer[p_offset..].as_ptr(), dest, p_filesz);
                    }
                }
                if p_memsz > p_filesz {
                    unsafe {
                        core::ptr::write_bytes(dest.add(p_filesz), 0, p_memsz - p_filesz);
                    }
                }
                let seg_end = dest_addr + p_memsz as u64;
                if seg_end > kernel_end {
                    kernel_end = seg_end;
                }
            }
        }

        let entry = if e_entry >= kernel_start {
            e_entry
        } else {
            kernel_start + e_entry
        };
        Some((kernel_start, kernel_end, entry))
    } else {
        None
    }
}

fn load_initramfs() -> Option<(u64, u64)> {
    let handle = uefi::boot::get_handle_for_protocol::<SimpleFileSystem>().ok()?;
    let mut fs = uefi::boot::open_protocol_exclusive::<SimpleFileSystem>(handle).ok()?;

    let mut root_dir = fs.open_volume().ok()?;
    let path = cstr16!("\\EFI\\ZUUPOS\\INITRAMFS.CPIO");
    let mut initrd_file = root_dir
        .open(path, FileMode::Read, FileAttribute::empty())
        .ok()?;

    if let FileType::Regular(mut file) = initrd_file.into_type().ok()? {
        let mut info_buffer = [0u8; 256];
        let file_size = file
            .get_info::<FileInfo>(&mut info_buffer)
            .ok()?
            .file_size() as usize;

        // Allocate physical memory for initramfs
        let initramfs_start = alloc_physical_memory(file_size)?;
        let mut buffer = alloc::vec![0u8; file_size];
        file.read(&mut buffer).ok()?;

        // Copy initramfs to physical memory
        unsafe {
            core::ptr::copy_nonoverlapping(buffer.as_ptr(), initramfs_start as *mut u8, file_size);
        }

        Some((initramfs_start, file_size as u64))
    } else {
        None
    }
}

fn alloc_physical_memory(size: usize) -> Option<u64> {
    let pages_needed = (size + 4095) / 4096;
    let phys_addr = uefi::boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        pages_needed,
    ).ok()?;
    Some(phys_addr.as_ptr() as u64)
}

fn setup_page_tables(
    _kernel_start: u64,
    _kernel_end: u64,
    _fb_addr: u64,
    _fb_width: u32,
    _fb_height: u32,
    _fb_stride: u32,
) -> Option<(u64, u64)> {
    const PAGE_SIZE: u64 = 4096;
    const KERNEL_BASE: u64 = 0xFFFF_8000_0000_0000;

    // Allocate page tables: PML4, PDPT, and 4 Page Directories for 4 GiB
    let pml4_addr = alloc_page()?;
    let pdpt_addr = alloc_page()?;
    let mut pd_addrs = [0u64; 4];
    for k in 0..4 {
        pd_addrs[k] = alloc_page()?;
    }

    // Zero PML4 and PDPT
    unsafe {
        core::ptr::write_bytes(pml4_addr as *mut u8, 0, PAGE_SIZE as usize);
        core::ptr::write_bytes(pdpt_addr as *mut u8, 0, PAGE_SIZE as usize);
    }

    // Map each PD with 512 entries of 2 MiB huge pages (0..4 GiB total)
    for (k, &pd_addr) in pd_addrs.iter().enumerate() {
        unsafe {
            core::ptr::write_bytes(pd_addr as *mut u8, 0, PAGE_SIZE as usize);
            let pd = pd_addr as *mut u64;
            let gig_offset = k as u64 * 1024 * 1024 * 1024;
            for i in 0..512usize {
                let phys = gig_offset + (i as u64 * 2 * 1024 * 1024);
                // Present (0x1) | Writable (0x2) | HugePage 2MiB (0x80)
                *pd.add(i) = phys | 0x83;
            }
        }
    }

    // Wire PDPT entries 0..4 to each PD
    let pdpt = pdpt_addr as *mut u64;
    for (k, &pd_addr) in pd_addrs.iter().enumerate() {
        unsafe {
            *pdpt.add(k) = pd_addr | 0x3;
        }
    }

    // Wire PML4:
    // PML4[0] -> PDPT (identity map first 4 GiB)
    // PML4[256] -> PDPT (higher-half direct map at 0xFFFF_8000_0000_0000)
    // PML4[511] -> PDPT (top higher-half map)
    let pml4 = pml4_addr as *mut u64;
    unsafe {
        *pml4.add(0) = pdpt_addr | 0x3;
        *pml4.add(256) = pdpt_addr | 0x3;
        *pml4.add(511) = pdpt_addr | 0x3;
    }

    Some((pml4_addr, KERNEL_BASE))
}

fn alloc_page() -> Option<u64> {
    let ptr = uefi::boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1,
    ).ok()?;
    Some(ptr.as_ptr() as u64)
}
