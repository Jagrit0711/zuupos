//! Inter-process communication: capability-based message passing.
//! Phase 1.3: Capability-based channels with rights.

use alloc::collections::{BTreeMap, VecDeque};
use core::sync::atomic::{AtomicU64, Ordering};
use spinning_top::Spinlock;
use crate::sched::current_pid;

pub const MAX_PAYLOAD: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityRight {
    Send = 1 << 0,
    Recv = 1 << 1,
    Both = (1 << 0) | (1 << 1),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChannelId(pub u64);

static NEXT_CHANNEL_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capability {
    pub channel_id: ChannelId,
    pub rights: CapabilityRight,
}

#[derive(Debug)]
pub struct Message {
    pub sender_pid: u32,
    pub payload: alloc::vec::Vec<u8>,
}

#[derive(Debug)]
pub struct Channel {
    pub id: ChannelId,
    pub queue: Spinlock<VecDeque<Message>>,
    pub rights: Spinlock<BTreeMap<u32, CapabilityRight>>, // pid -> rights
}

impl Channel {
    pub fn new() -> Self {
        let id = ChannelId(NEXT_CHANNEL_ID.fetch_add(1, Ordering::Relaxed));
        Self {
            id,
            queue: Spinlock::new(VecDeque::new()),
            rights: Spinlock::new(BTreeMap::new()),
        }
    }

    pub fn grant(&self, pid: u32, rights: CapabilityRight) {
        self.rights.lock().insert(pid, rights);
    }

    pub fn revoke(&self, pid: u32) {
        self.rights.lock().remove(&pid);
    }

    pub fn check_rights(&self, pid: u32, required: CapabilityRight) -> bool {
        self.rights.lock()
            .get(&pid)
            .map(|r| (*r as u8 & required as u8) == required as u8)
            .unwrap_or(false)
    }

    pub fn send(&self, msg: Message) -> Result<(), &'static str> {
        let pid = current_pid().ok_or("No current process")?;
        if !self.check_rights(pid, CapabilityRight::Send) {
            return Err("Capability denied: send");
        }
        if msg.payload.len() > MAX_PAYLOAD {
            return Err("Message too large");
        }
        self.queue.lock().push_back(msg);
        Ok(())
    }

    pub fn recv(&self) -> Result<Option<Message>, &'static str> {
        let pid = current_pid().ok_or("No current process")?;
        if !self.check_rights(pid, CapabilityRight::Recv) {
            return Err("Capability denied: recv");
        }
        Ok(self.queue.lock().pop_front())
    }
}

pub struct ChannelTable {
    channels: Spinlock<BTreeMap<ChannelId, alloc::sync::Arc<Channel>>>,
}

impl ChannelTable {
    pub fn new() -> Self {
        Self {
            channels: Spinlock::new(BTreeMap::new()),
        }
    }

    pub fn create(&self) -> alloc::sync::Arc<Channel> {
        let channel = alloc::sync::Arc::new(Channel::new());
        self.channels.lock().insert(channel.id, channel.clone());
        channel
    }

    pub fn get(&self, id: ChannelId) -> Option<alloc::sync::Arc<Channel>> {
        self.channels.lock().get(&id).cloned()
    }

    pub fn remove(&self, id: ChannelId) {
        self.channels.lock().remove(&id);
    }
}

// Global channel table
static CHANNEL_TABLE: Spinlock<Option<ChannelTable>> = Spinlock::new(None);

pub fn init() {
    *CHANNEL_TABLE.lock() = Some(ChannelTable::new());
}

pub fn channel_table() -> Option<ChannelTable> {
    CHANNEL_TABLE.lock().as_ref().map(|_| ChannelTable { channels: Spinlock::new(BTreeMap::new()) })
}

pub fn create_channel() -> Result<ChannelId, &'static str> {
    let mut guard = CHANNEL_TABLE.lock();
    if guard.is_none() {
        *guard = Some(ChannelTable::new());
    }
    let table = guard.as_ref().unwrap();
    let channel = table.create();
    if let Some(pid) = current_pid() {
        channel.grant(pid, CapabilityRight::Both);
    }
    Ok(channel.id)
}

pub fn send_message(channel_id: u64, payload: alloc::vec::Vec<u8>) -> Result<(), &'static str> {
    let guard = CHANNEL_TABLE.lock();
    if let Some(ref table) = *guard {
        if let Some(channel) = table.get(ChannelId(channel_id)) {
            let pid = current_pid().unwrap_or(0);
            return channel.send(Message { sender_pid: pid, payload });
        }
    }
    Err("Channel not found")
}

pub fn recv_message(channel_id: u64) -> Result<Option<Message>, &'static str> {
    let guard = CHANNEL_TABLE.lock();
    if let Some(ref table) = *guard {
        if let Some(channel) = table.get(ChannelId(channel_id)) {
            return channel.recv();
        }
    }
    Err("Channel not found")
}