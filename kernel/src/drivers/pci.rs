//! PCI enumeration and device driver framework.

use super::super::mm::virt::{map, phys_to_virt};
use alloc::vec::Vec;
use spinning_top::Spinlock;
use x86_64::instructions::port::Port;
use x86_64::{PhysAddr, VirtAddr};

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PciAddress {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub address: PciAddress,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision_id: u8,
    pub header_type: u8,
    pub bar: [u64; 6],
    pub irq: u8,
    pub irq_pin: u8,
}

static PCI_DEVICES: Spinlock<Vec<PciDevice>> = Spinlock::new(Vec::new());

pub fn init() {
    log::info!("[pci] Enumerating PCI bus...");
    enumerate_bus(0);
    let count = PCI_DEVICES.lock().len();
    log::info!("[pci] Found {} device(s)", count);
    for dev in PCI_DEVICES.lock().iter() {
        log::info!(
            "[pci] {:02x}:{:02x}.{:x} [{:04x}:{:04x}] class={:02x}{:02x}{:02x} IRQ={}",
            dev.address.bus,
            dev.address.device,
            dev.address.function,
            dev.vendor_id,
            dev.device_id,
            dev.class_code,
            dev.subclass,
            dev.prog_if,
            dev.irq
        );
    }
}

fn enumerate_bus(bus: u8) {
    for device in 0..32 {
        for function in 0..8 {
            let addr = PciAddress {
                bus,
                device,
                function,
            };
            let vendor_id = read_config(addr, 0x00);
            if vendor_id == 0xFFFF {
                if function == 0 {
                    break;
                }
                continue;
            }

            let device_id = read_config(addr, 0x02);
            let class_code = (read_config(addr, 0x0B) >> 8) as u8;
            let subclass = (read_config(addr, 0x0A) >> 8) as u8;
            let prog_if = (read_config(addr, 0x09) >> 8) as u8;
            let revision_id = read_config(addr, 0x08) as u8;
            let header_type = (read_config(addr, 0x0E) >> 8) as u8;

            let mut bar = [0u64; 6];
            let mut i = 0usize;
            while i < 6 {
                let bar_reg = (0x10 + i * 4) as u8;
                let bar_val = read_config(addr, bar_reg);
                if bar_val != 0 {
                    let is_64bit = (bar_val & 0x4) != 0;
                    let is_io = (bar_val & 0x1) != 0;
                    if is_io {
                        bar[i] = (bar_val & 0xFFFFFFFC) as u64;
                    } else if is_64bit {
                        let bar_high = read_config(addr, bar_reg + 4);
                        bar[i] = ((bar_high as u64) << 32) | (bar_val & 0xFFFFFFF0) as u64;
                        i += 1;
                    } else {
                        bar[i] = (bar_val & 0xFFFFFFF0) as u64;
                    }
                }
                i += 1;
            }

            let irq_pin = (read_config(addr, 0x3D) >> 8) as u8;
            let irq_line = read_config(addr, 0x3C) as u8;

            let dev = PciDevice {
                address: addr,
                vendor_id: vendor_id as u16,
                device_id: device_id as u16,
                class_code,
                subclass,
                prog_if,
                revision_id,
                header_type,
                bar,
                irq: irq_line,
                irq_pin,
            };

            PCI_DEVICES.lock().push(dev);

            if header_type & 0x80 != 0 || function == 0 {
                if header_type & 0x7F == 1 {
                    // PCI-to-PCI bridge
                    let secondary_bus = (read_config(addr, 0x19) >> 8) as u8;
                    enumerate_bus(secondary_bus);
                }
            }
        }
    }
}

fn read_config(addr: PciAddress, offset: u8) -> u32 {
    let address = 0x8000_0000u32
        | ((addr.bus as u32) << 16)
        | ((addr.device as u32) << 11)
        | ((addr.function as u32) << 8)
        | ((offset & 0xFC) as u32);

    let mut port_addr: Port<u32> = Port::new(PCI_CONFIG_ADDRESS);
    let mut port_data: Port<u32> = Port::new(PCI_CONFIG_DATA);

    unsafe {
        port_addr.write(address);
        port_data.read()
    }
}

pub fn write_config(addr: PciAddress, offset: u8, value: u32) {
    let address = 0x8000_0000u32
        | ((addr.bus as u32) << 16)
        | ((addr.device as u32) << 11)
        | ((addr.function as u32) << 8)
        | ((offset & 0xFC) as u32);

    let mut port_addr: Port<u32> = Port::new(PCI_CONFIG_ADDRESS);
    let mut port_data: Port<u32> = Port::new(PCI_CONFIG_DATA);

    unsafe {
        port_addr.write(address);
        port_data.write(value);
    }
}

pub fn find_device(class_code: u8, subclass: u8) -> Option<PciDevice> {
    PCI_DEVICES
        .lock()
        .iter()
        .find(|d| d.class_code == class_code && d.subclass == subclass)
        .copied()
}

pub fn find_device_by_id(vendor_id: u16, device_id: u16) -> Option<PciDevice> {
    PCI_DEVICES
        .lock()
        .iter()
        .find(|d| d.vendor_id == vendor_id && d.device_id == device_id)
        .copied()
}

pub fn get_all_devices() -> Vec<PciDevice> {
    PCI_DEVICES.lock().clone()
}

pub fn map_bar(dev: &PciDevice, bar_index: usize) -> Option<VirtAddr> {
    if bar_index >= 6 {
        return None;
    }
    let phys = dev.bar[bar_index];
    if phys == 0 {
        return None;
    }

    // Map the BAR
    let size = 4096; // Default page size, real impl should query size
    let virt = phys_to_virt(phys);
    map(phys, virt, size, 0x3); // PRESENT | WRITABLE
    Some(VirtAddr::new(virt))
}
