//! Programmable Interval Timer (PIT) for x86-64.
//! Generates periodic timer interrupts for preemptive scheduling.

use x86_64::instructions::port::Port;

const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const PIT_FREQUENCY: u32 = 1193182; // 1.193182 MHz
const TIMER_FREQUENCY: u32 = 1000; // 1000 Hz (1 ms)

pub fn init() {
    let divisor = PIT_FREQUENCY / TIMER_FREQUENCY;
    let divisor_low = (divisor & 0xFF) as u8;
    let divisor_high = ((divisor >> 8) & 0xFF) as u8;

    // Command byte: channel 0, lobyte/hibyte, rate generator (mode 3), binary
    let mut cmd_port: Port<u8> = Port::new(PIT_COMMAND);
    unsafe { cmd_port.write(0x36) };

    let mut data_port: Port<u8> = Port::new(PIT_CHANNEL0);
    unsafe { data_port.write(divisor_low) };
    unsafe { data_port.write(divisor_high) };

    log::info!("[pit] PIT initialized at {} Hz (divisor={})", TIMER_FREQUENCY, divisor);
}