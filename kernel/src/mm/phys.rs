//! Physical frame allocator using a buddy system.
//! Initialized from the UEFI memory map provided by the bootloader.

use super::super::arch::x86_64::boot_info::BootInfo;
use spinning_top::Spinlock;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};

const PAGE_SIZE: u64 = 4096;
const MAX_ORDER: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemoryType {
    Reserved = 0,
    Available = 1,
    BootServicesCode = 2,
    BootServicesData = 3,
    LoaderCode = 4,
    LoaderData = 5,
    MemoryMappedIO = 6,
    MemoryMappedIOPortSpace = 7,
    PalCode = 8,
    PersistentMemory = 9,
    UnusableMemory = 10,
    AcpiReclaim = 11,
    AcpiNvs = 12,
    AcpiMemoryNVS = 13,
}

#[repr(C, packed)]
struct UefiMemoryDescriptor {
    _type: u32,
    _padding: u32,
    physical_start: u64,
    virtual_start: u64,
    number_of_pages: u64,
    _attribute: u64,
}

static PHYS_ALLOCATOR: Spinlock<Option<BuddyAllocator>> = Spinlock::new(None);

pub struct BuddyAllocator {
    free_lists: [alloc::collections::VecDeque<u64>; MAX_ORDER + 1],
    total_frames: u64,
    used_frames: u64,
    max_addr: u64,
}

impl BuddyAllocator {
    fn new() -> Self {
        let free_lists: [alloc::collections::VecDeque<u64>; MAX_ORDER + 1] = core::array::from_fn(|_| alloc::collections::VecDeque::new());
        Self {
            free_lists,
            total_frames: 0,
            used_frames: 0,
            max_addr: 0,
        }
    }

    fn add_region(&mut self, base: u64, pages: u64) {
        let mut addr = base;
        let mut remaining = pages;

        while remaining > 0 {
            let max_order = (remaining.trailing_zeros() as usize).min(MAX_ORDER);
            let align_order = (addr / PAGE_SIZE).trailing_zeros() as usize;
            let order = max_order.min(align_order).min(MAX_ORDER);

            let frames = 1u64 << order;
            if frames > remaining {
                let order = (remaining as u64).next_power_of_two().trailing_zeros() as usize - 1;
                let frames = 1u64 << order;
                self.free_lists[order].push_back(addr);
                addr += frames * PAGE_SIZE;
                remaining -= frames;
            } else {
                self.free_lists[order].push_back(addr);
                addr += frames * PAGE_SIZE;
                remaining -= frames;
            }
        }
    }

    fn alloc(&mut self, order: usize) -> Option<u64> {
        if order > MAX_ORDER {
            return None;
        }

        for i in order..=MAX_ORDER {
            if let Some(addr) = self.free_lists[i].pop_front() {
                if i > order {
                    let mut split_addr = addr;
                    for j in (order + 1..=i).rev() {
                        let half = 1u64 << (j - 1);
                        let buddy = split_addr + half * PAGE_SIZE;
                        self.free_lists[j - 1].push_back(buddy);
                        split_addr += PAGE_SIZE;
                    }
                }
                let frames = 1u64 << order;
                self.used_frames += frames;
                return Some(addr);
            }
        }
        None
    }

    fn free(&mut self, addr: u64, order: usize) {
        if order > MAX_ORDER {
            return;
        }

        let mut current_addr = addr;
        let mut current_order = order;

        loop {
            let buddy_addr = current_addr ^ (1u64 << current_order) * PAGE_SIZE;

            if current_order >= MAX_ORDER {
                self.free_lists[current_order].push_back(current_addr);
                self.used_frames -= 1u64 << current_order;
                break;
            }

            let buddy_list = &mut self.free_lists[current_order];
            if let Some(pos) = buddy_list.iter().position(|&x| x == buddy_addr) {
                buddy_list.remove(pos);
                current_addr = current_addr.min(buddy_addr);
                current_order += 1;
            } else {
                self.free_lists[current_order].push_back(current_addr);
                self.used_frames -= 1u64 << current_order;
                break;
            }
        }
    }

    fn alloc_frames(&mut self, n: usize) -> Option<u64> {
        if n == 0 {
            return None;
        }
        let order = (n as u64).next_power_of_two().trailing_zeros() as usize - 1;
        self.alloc(order)
    }

    fn free_frames(&mut self, addr: u64, n: usize) {
        if n == 0 {
            return;
        }
        let order = (n as u64).next_power_of_two().trailing_zeros() as usize - 1;
        self.free(addr, order);
    }
}

pub fn init(boot_info: &BootInfo) {
    let mut allocator = BuddyAllocator::new();

    let mmap_addr = boot_info.memory_map_addr as *const UefiMemoryDescriptor;
    let desc_size = boot_info.memory_map_desc_size;
    let entry_count = boot_info.memory_map_len / desc_size;

    for i in 0..entry_count {
        let desc = unsafe { &*mmap_addr.add(i) };
        let mem_type = match desc._type {
            1 => MemoryType::Available,
            2 => MemoryType::BootServicesCode,
            3 => MemoryType::BootServicesData,
            4 => MemoryType::LoaderCode,
            5 => MemoryType::LoaderData,
            7 => MemoryType::AcpiReclaim,
            _ => MemoryType::Reserved,
        };

        if matches!(mem_type, MemoryType::Available
            | MemoryType::BootServicesCode
            | MemoryType::BootServicesData
            | MemoryType::LoaderCode
            | MemoryType::LoaderData
            | MemoryType::AcpiReclaim) {
            let base = desc.physical_start;
            let pages = desc.number_of_pages;
            allocator.add_region(base, pages);
            allocator.total_frames += pages;
            allocator.max_addr = allocator.max_addr.max(base + pages * PAGE_SIZE);
        }
    }

    log::info!(
        "[mm::phys] Physical memory: {} MiB total, {} MiB usable ({} frames)",
        allocator.max_addr / (1024 * 1024),
        allocator.total_frames * PAGE_SIZE / (1024 * 1024),
        allocator.total_frames
    );

    *PHYS_ALLOCATOR.lock() = Some(allocator);
}

pub fn alloc_frames(n: usize) -> Option<u64> {
    PHYS_ALLOCATOR.lock().as_mut().and_then(|a| a.alloc_frames(n))
}

pub fn free_frames(base: u64, n: usize) {
    if let Some(allocator) = PHYS_ALLOCATOR.lock().as_mut() {
        allocator.free_frames(base, n);
    }
}

pub fn total_frames() -> u64 {
    PHYS_ALLOCATOR.lock().as_ref().map(|a| a.total_frames).unwrap_or(0)
}

pub fn used_frames() -> u64 {
    PHYS_ALLOCATOR.lock().as_ref().map(|a| a.used_frames).unwrap_or(0)
}

unsafe impl FrameAllocator<Size4KiB> for BuddyAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.alloc(0).map(|addr| PhysFrame::containing_address(x86_64::PhysAddr::new(addr)))
    }
}
