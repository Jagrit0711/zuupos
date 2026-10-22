//! x86-64 architecture initialization.

pub mod acpi;
pub mod boot_info;
pub mod gdt;
pub mod idt;
pub mod tss;

pub fn init() {
    gdt::init();
    idt::init();
    tss::init();
}
