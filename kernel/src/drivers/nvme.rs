//! NVMe (Non-Volatile Memory Express) driver skeleton.
//! Provides basic block device access for NVMe SSDs.

use super::pci::{PciDevice, map_bar};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};
use spinning_top::Spinlock;
use x86_64::VirtAddr;

const NVME_CLASS_CODE: u8 = 0x01;
const NVME_SUBCLASS: u8 = 0x08;

#[repr(C)]
#[derive(Clone, Copy)]
struct NvmeQueueEntry {
    dw0: u32,
    dw1: u32,
    dw2: u32,
    dw3: u32,
    dw4: u32,
    dw5: u32,
    dw6: u32,
    dw7: u32,
    dw8: u32,
    dw9: u32,
    dw10: u32,
    dw11: u32,
    dw12: u32,
    dw13: u32,
    dw14: u32,
    dw15: u32,
}

struct NvmeQueue {
    entries: Vec<NvmeQueueEntry>,
    head: AtomicU32,
    tail: AtomicU32,
    size: u32,
    doorbell_offset: u32,
}

impl NvmeQueue {
    fn new(size: u32, doorbell_offset: u32) -> Self {
        let mut entries = Vec::with_capacity(size as usize);
        for _ in 0..size {
            entries.push(NvmeQueueEntry {
                dw0: 0, dw1: 0, dw2: 0, dw3: 0,
                dw4: 0, dw5: 0, dw6: 0, dw7: 0,
                dw8: 0, dw9: 0, dw10: 0, dw11: 0,
                dw12: 0, dw13: 0, dw14: 0, dw15: 0,
            });
        }
        Self {
            entries,
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            size,
            doorbell_offset,
        }
    }

    fn submit(&mut self, entry: NvmeQueueEntry) -> bool {
        let tail = self.tail.load(Ordering::Acquire);
        let next_tail = (tail + 1) % self.size;
        if next_tail == self.head.load(Ordering::Acquire) {
            return false; // Queue full
        }
        self.entries[tail as usize] = entry;
        self.tail.store(next_tail, Ordering::Release);
        true
    }

    fn complete(&mut self) -> Option<NvmeQueueEntry> {
        let head = self.head.load(Ordering::Acquire);
        if head == self.tail.load(Ordering::Acquire) {
            return None; // Queue empty
        }
        let entry = self.entries[head as usize];
        self.head.store((head + 1) % self.size, Ordering::Release);
        Some(entry)
    }
}

#[repr(C)]
struct NvmeRegisters {
    pub cap: u64,           // 0x00 - Controller Capabilities
    pub vs: u32,            // 0x08 - Version
    pub intms: u32,         // 0x0C - Interrupt Mask Set
    pub intmc: u32,         // 0x10 - Interrupt Mask Clear
    pub cc: u32,            // 0x14 - Controller Configuration
    _reserved1: u32,        // 0x18
    pub csts: u32,          // 0x1C - Controller Status
    pub nssr: u32,          // 0x20 - NVM Subsystem Reset
    pub aqa: u32,           // 0x24 - Admin Queue Attributes
    pub asq: u64,           // 0x28 - Admin Submission Queue Base Address
    pub acq: u64,           // 0x30 - Admin Completion Queue Base Address
    // ... more registers
}

pub struct NvmeController {
    regs: &'static mut NvmeRegisters,
    admin_sq: NvmeQueue,
    admin_cq: NvmeQueue,
    io_queues: Vec<NvmeQueue>,
    device: PciDevice,
}

impl NvmeController {
    pub fn init(device: PciDevice) -> Option<Self> {
        log::info!("[nvme] Initializing NVMe controller {:04x}:{:04x}", device.vendor_id, device.device_id);

        let bar0_virt = map_bar(&device, 0)?;
        let regs = unsafe { &mut *(bar0_virt.as_u64() as *mut NvmeRegisters) };

        // Check controller readiness
        if regs.csts & 0x1 != 0 {
            log::warn!("[nvme] Controller not ready (CSTS.RDY=1)");
        }

        // Disable controller
        regs.cc = 0;
        while regs.csts & 0x1 != 0 {
            core::hint::spin_loop();
        }

        // Allocate admin queues (4KB each, 32 entries)
        let admin_sq = NvmeQueue::new(32, 0x1000);
        let admin_cq = NvmeQueue::new(32, 0x1000 + 64); // Completion queue doorbells typically offset

        // Set queue addresses (physical addresses)
        // In real implementation, need to get physical addresses of queues
        regs.aqa = ((32 - 1) << 16) | (32 - 1); // ACQS=31, ASQS=31 (0-based)
        regs.asq = 0; // Physical address of admin submission queue
        regs.acq = 0; // Physical address of admin completion queue

        // Enable controller with I/O queues
        regs.cc = 0x1 | (1 << 4) | (1 << 8); // EN=1, IOQES=4 (64B), IOSQES=6 (64B)

        // Wait for ready
        while regs.csts & 0x1 == 0 {
            core::hint::spin_loop();
        }

        log::info!("[nvme] Controller ready! Version: {:08x}", regs.vs);

        Some(Self {
            regs,
            admin_sq,
            admin_cq,
            io_queues: Vec::new(),
            device,
        })
    }

    pub fn identify(&mut self) -> Option<Vec<u8>> {
        // Submit Admin Identify command (opcode 0x06)
        let entry = NvmeQueueEntry {
            dw0: 0x06, // OPC=0x06 (Identify), CID=0
            dw1: 0,
            dw2: 0,
            dw3: 0,
            dw4: 0,
            dw5: 0,
            dw6: 0,
            dw7: 0,
            dw8: 0,
            dw9: 0,
            dw10: 1, // CNS=1 (Controller)
            dw11: 0,
            dw12: 0,
            dw13: 0,
            dw14: 0,
            dw15: 0,
        };
        self.admin_sq.submit(entry);

        // Ring doorbell
        // In real impl: write to doorbell register

        // Wait for completion
        while let Some(_) = self.admin_cq.complete() {
            // Process completion
        }

        None // Stub
    }

    pub fn read_blocks(&mut self, _lba: u64, _count: u32, _buf: &mut [u8]) -> Result<(), &'static str> {
        Err("Not implemented")
    }

    pub fn write_blocks(&mut self, _lba: u64, _count: u32, _buf: &[u8]) -> Result<(), &'static str> {
        Err("Not implemented")
    }
}

static NVME_CONTROLLER: Spinlock<Option<NvmeController>> = Spinlock::new(None);

pub fn init() {
    if let Some(device) = super::pci::find_device(NVME_CLASS_CODE, NVME_SUBCLASS) {
        log::info!("[nvme] Found NVMe device: {:04x}:{:04x}", device.vendor_id, device.device_id);
        if let Some(controller) = NvmeController::init(device) {
            *NVME_CONTROLLER.lock() = Some(controller);
            log::info!("[nvme] NVMe driver initialized");
        }
    } else {
        log::info!("[nvme] No NVMe devices found");
    }
}

pub fn read_blocks(lba: u64, count: u32, buf: &mut [u8]) -> Result<(), &'static str> {
    if let Some(controller) = NVME_CONTROLLER.lock().as_mut() {
        controller.read_blocks(lba, count, buf)
    } else {
        Err("NVMe not initialized")
    }
}

pub fn write_blocks(lba: u64, count: u32, buf: &[u8]) -> Result<(), &'static str> {
    if let Some(controller) = NVME_CONTROLLER.lock().as_mut() {
        controller.write_blocks(lba, count, buf)
    } else {
        Err("NVMe not initialized")
    }
}