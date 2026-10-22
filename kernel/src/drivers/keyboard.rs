//! PS/2 keyboard scancode handler.
//! Scancodes are sent to the agent event queue.

use spinning_top::Spinlock;
use alloc::collections::VecDeque;

static SCANCODE_QUEUE: Spinlock<VecDeque<u8>> = Spinlock::new(VecDeque::new());

pub fn handle_scancode(scancode: u8) {
    SCANCODE_QUEUE.lock().push_back(scancode);
    // TODO: Wake the agent event loop when input arrives.
}

pub fn pop_scancode() -> Option<u8> {
    SCANCODE_QUEUE.lock().pop_front()
}
