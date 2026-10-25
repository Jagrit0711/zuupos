//! Kernel heap allocator (linked list, then replace with slab).

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initial kernel heap: 32 MiB.
const HEAP_SIZE: usize = 32 * 1024 * 1024;

static mut HEAP_SPACE: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];

pub fn init() {
    unsafe {
        ALLOCATOR.lock().init(HEAP_SPACE.as_mut_ptr(), HEAP_SIZE);
    }
    log::debug!("[mm::heap] {}MiB heap initialized", HEAP_SIZE / 1024 / 1024);
}
