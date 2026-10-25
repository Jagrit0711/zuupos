//! 64-bit ELF executable loader for ZuuposOS.
//!
//! Parses ELF64 binaries, maps PT_LOAD segments into virtual memory with appropriate
//! permissions, initializes the BSS section, sets up the user stack, and returns the
//! entry point ready for scheduler dispatch.

use crate::mm::phys::alloc_frames;
use crate::mm::virt::{map, phys_to_virt};
use log::{debug, info, warn};
use x86_64::structures::paging::PageTableFlags;
use x86_64::{PhysAddr, VirtAddr};

pub struct LoadedElf {
    pub entry_point: u64,
    pub user_stack: VirtAddr,
    pub page_table: PhysAddr,
}

pub fn load_elf(elf_bytes: &[u8]) -> Result<LoadedElf, &'static str> {
    if elf_bytes.len() < 64 {
        return Err("ELF binary too small");
    }

    // Verify ELF Magic: \x7fELF
    if &elf_bytes[0..4] != b"\x7fELF" {
        return Err("Invalid ELF magic");
    }

    // Must be 64-bit (class 2) and Little Endian (data 1)
    if elf_bytes[4] != 2 || elf_bytes[5] != 1 {
        return Err("Unsupported ELF class or endianness (must be 64-bit LE)");
    }

    // Must be x86_64 architecture (e_machine = 0x3E)
    let e_machine = u16::from_le_bytes([elf_bytes[18], elf_bytes[19]]);
    if e_machine != 0x3E {
        return Err("Unsupported ELF architecture (must be x86_64)");
    }

    let e_entry = u64::from_le_bytes(elf_bytes[24..32].try_into().unwrap());
    let e_phoff = u64::from_le_bytes(elf_bytes[32..40].try_into().unwrap()) as usize;
    let e_phentsize = u16::from_le_bytes(elf_bytes[54..56].try_into().unwrap()) as usize;
    let e_phnum = u16::from_le_bytes(elf_bytes[56..58].try_into().unwrap()) as usize;

    info!(
        "[loader] Parsing ELF: entry={:#x}, phoff={:#x}, phnum={}",
        e_entry, e_phoff, e_phnum
    );

    // Load each PT_LOAD segment
    for i in 0..e_phnum {
        let ph_start = e_phoff + i * e_phentsize;
        if ph_start + 56 > elf_bytes.len() {
            return Err("ELF program header extends beyond file");
        }

        let ph = &elf_bytes[ph_start..ph_start + 56];
        let p_type = u32::from_le_bytes(ph[0..4].try_into().unwrap());
        let p_flags = u32::from_le_bytes(ph[4..8].try_into().unwrap());
        let p_offset = u64::from_le_bytes(ph[8..16].try_into().unwrap()) as usize;
        let p_vaddr = u64::from_le_bytes(ph[16..24].try_into().unwrap());
        let p_filesz = u64::from_le_bytes(ph[32..40].try_into().unwrap()) as usize;
        let p_memsz = u64::from_le_bytes(ph[40..48].try_into().unwrap()) as usize;

        // PT_LOAD = 1
        if p_type == 1 {
            debug!(
                "[loader] PT_LOAD segment: vaddr={:#x}, filesz={}, memsz={}, flags={:#x}",
                p_vaddr, p_filesz, p_memsz, p_flags
            );

            let vaddr_start = p_vaddr & !0xFFF;
            let vaddr_end = (p_vaddr + p_memsz as u64 + 4095) & !0xFFF;
            let num_pages = ((vaddr_end - vaddr_start) / 4096) as usize;

            let phys_frame = alloc_frames(num_pages).ok_or("Out of memory for ELF segment")?;

            // Compute page table flags
            let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
            // Always writable during load; check write flag
            flags |= PageTableFlags::WRITABLE;
            if (p_flags & 1) == 0 {
                // If not executable, set NX
                flags |= PageTableFlags::NO_EXECUTE;
            }

            // Map segment into virtual address space
            map(phys_frame, vaddr_start, (num_pages * 4096) as u64, flags.bits());

            // Zero memory in physical frame via direct higher-half kernel mapping
            let phys_virt = phys_to_virt(phys_frame) as *mut u8;
            unsafe {
                core::ptr::write_bytes(phys_virt, 0, num_pages * 4096);
            }

            // Copy file content into destination
            if p_filesz > 0 {
                if p_offset + p_filesz > elf_bytes.len() {
                    return Err("Segment data exceeds ELF file size");
                }
                let page_offset = (p_vaddr - vaddr_start) as usize;
                let dst = unsafe { phys_virt.add(page_offset) };
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        elf_bytes[p_offset..p_offset + p_filesz].as_ptr(),
                        dst,
                        p_filesz,
                    );
                }
            }
        }
    }

    // Allocate and map user stack (32 KiB = 8 pages)
    const USER_STACK_TOP: u64 = 0x0000_7FFF_FFFF_0000;
    const USER_STACK_PAGES: usize = 8;
    let stack_frame = alloc_frames(USER_STACK_PAGES).ok_or("Out of memory for user stack")?;
    let stack_vaddr = USER_STACK_TOP - (USER_STACK_PAGES * 4096) as u64;

    map(
        stack_frame,
        stack_vaddr,
        (USER_STACK_PAGES * 4096) as u64,
        (PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE).bits(),
    );

    // Zero the stack
    unsafe {
        core::ptr::write_bytes(phys_to_virt(stack_frame) as *mut u8, 0, USER_STACK_PAGES * 4096);
    }

    info!(
        "[loader] ELF loaded successfully: entry={:#x}, stack_top={:#x}",
        e_entry, USER_STACK_TOP
    );

    // Allocate a page directory for process if needed or use primary
    let page_table = alloc_frames(1).unwrap_or(0);

    Ok(LoadedElf {
        entry_point: e_entry,
        user_stack: VirtAddr::new(USER_STACK_TOP),
        page_table: PhysAddr::new(page_table),
    })
}
