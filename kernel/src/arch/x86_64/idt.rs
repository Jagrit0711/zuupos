//! Interrupt Descriptor Table (IDT) for x86-64.

use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spinning_top::Spinlock;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Spinlock<ChainedPics> =
    Spinlock::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.general_protection_fault.set_handler_fn(gpf_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        // Timer IRQ (IRQ0 → vector 0x20 / 32)
        idt[PIC_1_OFFSET].set_handler_fn(timer_handler);
        // Keyboard IRQ (IRQ1 → vector 0x21 / 33)
        idt[PIC_1_OFFSET + 1].set_handler_fn(keyboard_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
    unsafe { PICS.lock().initialize() };
}

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    log::debug!("EXCEPTION: BREAKPOINT\n{:#?}", frame);
}

extern "x86-interrupt" fn gpf_handler(frame: InterruptStackFrame, error_code: u64) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT (code={:#x})\n{:#?}",
        error_code, frame
    );
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", frame);
}

extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    panic!(
        "EXCEPTION: PAGE FAULT\n  Accessed address: {:?}\n  Error code: {:?}\n{:#?}",
        Cr2::read(),
        error_code,
        frame,
    );
}

// Timer handler wrapper that calls the naked function in sched
extern "x86-interrupt" fn timer_handler(mut frame: InterruptStackFrame) {
    unsafe {
        crate::sched::timer_handler_entry(&mut frame as *mut InterruptStackFrame);
    }
}

extern "x86-interrupt" fn keyboard_handler(_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    let mut port: Port<u8> = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::drivers::keyboard::handle_scancode(scancode);
    unsafe { PICS.lock().notify_end_of_interrupt(PIC_1_OFFSET + 1) };
}
