//! Inter-process communication: capability-based message passing.
//! Phase 1.3: Basic synchronous message channels.

use alloc::collections::VecDeque;
use spinning_top::Spinlock;

pub struct Message {
    pub sender_pid: u32,
    pub payload: alloc::vec::Vec<u8>,
}

pub struct Channel {
    queue: Spinlock<VecDeque<Message>>,
}

impl Channel {
    pub const fn new() -> Self {
        Self {
            queue: Spinlock::new(VecDeque::new()),
        }
    }

    pub fn send(&self, msg: Message) {
        self.queue.lock().push_back(msg);
    }

    pub fn recv(&self) -> Option<Message> {
        self.queue.lock().pop_front()
    }
}
