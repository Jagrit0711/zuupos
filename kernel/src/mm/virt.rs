//! Virtual memory manager: 4-level paging on x86-64.

use super::super::arch::x86_64::boot_info::BootInfo;
use super::phys::alloc_frames;
use spinning_top::Spinlock;
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
        Size4KiB,
    },
    VirtAddr,
};

const KERNEL_BASE: u64 = 0xFFFF_8000_0000_0000;
const KERNEL_OFFSET: u64 = 0xFFFF_8000_0000_0000;
const IDENTITY_MAP_LIMIT: u64 = 4 * 1024 * 1024 * 1024;

static VIRT_MAPPER: Spinlock<Option<OffsetPageTable<'static>>> = Spinlock::new(None);

pub fn init(boot_info: &BootInfo) {
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let level_4_table = unsafe { &mut *(boot_info.pml4_addr as *mut PageTable) };

    let mut mapper = unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset) };

    log::info!(
        "[mm::virt] Setting up higher-half kernel mapping at {:#x}",
        KERNEL_BASE
    );
    log::info!(
        "[mm::virt] Identity-mapping first {} GiB",
        IDENTITY_MAP_LIMIT / (1024 * 1024 * 1024)
    );

    map_kernel(boot_info, &mut mapper);
    identity_map_low_memory(&mut mapper);

    *VIRT_MAPPER.lock() = Some(mapper);

    log::info!("[mm::virt] Virtual memory initialized — higher-half kernel active");
}

fn map_kernel(boot_info: &BootInfo, mapper: &mut OffsetPageTable) {
    let kernel_start = boot_info.kernel_phys_start;
    let kernel_end = boot_info.kernel_phys_end;

    let start_page: Page<Size4KiB> =
        Page::containing_address(VirtAddr::new(KERNEL_BASE + kernel_start));
    let end_page: Page<Size4KiB> =
        Page::containing_address(VirtAddr::new(KERNEL_BASE + kernel_end));

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = PhysFrame::containing_address(x86_64::PhysAddr::new(
            kernel_start + (page.start_address().as_u64() - KERNEL_BASE),
        ));
        unsafe {
            mapper
                .map_to(page, frame, flags, &mut BuddyFrameAllocator)
                .expect("kernel map failed");
        }
    }
}

fn identity_map_low_memory(mapper: &mut OffsetPageTable) {
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    for page_addr in (0..IDENTITY_MAP_LIMIT).step_by(4096) {
        let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(page_addr));
        let frame = PhysFrame::containing_address(x86_64::PhysAddr::new(page_addr));
        unsafe {
            let _ = mapper.map_to(page, frame, flags, &mut BuddyFrameAllocator);
        }
    }
}

struct BuddyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for BuddyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        alloc_frames(1).map(|addr| PhysFrame::containing_address(x86_64::PhysAddr::new(addr)))
    }
}

/// Map a physical region into virtual space.
pub fn map(phys: u64, virt: u64, size: u64, flags: u64) {
    let pt_flags = PageTableFlags::from_bits_truncate(flags);
    let mut mapper_guard = VIRT_MAPPER.lock();

    if let Some(ref mut mapper) = *mapper_guard {
        let start_page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(virt));
        let end_page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(virt + size - 1));

        for page in Page::range_inclusive(start_page, end_page) {
            let frame = PhysFrame::containing_address(x86_64::PhysAddr::new(
                phys + (page.start_address().as_u64() - virt),
            ));
            unsafe {
                let _ = mapper.map_to(page, frame, pt_flags, &mut BuddyFrameAllocator);
            }
        }
    } else {
        log::warn!("[mm::virt] map() called before init");
    }
}

/// Get the kernel virtual address for a physical address.
pub fn phys_to_virt(phys: u64) -> u64 {
    KERNEL_OFFSET + phys
}

/// Get the physical address for a kernel virtual address.
pub fn virt_to_phys(virt: u64) -> Option<u64> {
    if virt >= KERNEL_BASE {
        Some(virt - KERNEL_BASE)
    } else {
        None
    }
}

pub fn get_mapper() -> Option<&'static OffsetPageTable<'static>> {
    VIRT_MAPPER.lock().as_ref().map(|m| {
        let ptr = m as *const _;
        unsafe { &*ptr }
    })
}

pub fn map_framebuffer(fb_addr: u64, fb_size: u64) {
    let virt_addr = KERNEL_BASE + fb_addr;
    map(
        fb_addr,
        virt_addr,
        fb_size,
        PageTableFlags::PRESENT.bits()
            | PageTableFlags::WRITABLE.bits()
            | PageTableFlags::NO_EXECUTE.bits(),
    );
    log::info!(
        "[mm::virt] Framebuffer mapped: {:#x} -> {:#x} ({} bytes)",
        fb_addr,
        virt_addr,
        fb_size
    );
}
