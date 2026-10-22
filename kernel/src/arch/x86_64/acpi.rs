//! ACPI initialization via RSDP pointer from bootloader.

use super::boot_info::BootInfo;

pub fn init(boot_info: &BootInfo) {
    // Phase 1.3: Parse ACPI tables for MADT (APICs), HPET, etc.
    // Using the `acpi` crate with a custom PhysicalMemoryHandler.
    let _rsdp_addr = boot_info.rsdp_addr;
    log::debug!("[acpi] RSDP at {:#x} (parsing deferred — Phase 1.3)", _rsdp_addr);
}
