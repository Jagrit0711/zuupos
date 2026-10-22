//! UART 16550 serial output — primary debug console.

use uart_16550::SerialPort;
use spinning_top::Spinlock;

static SERIAL1: Spinlock<SerialPort> = Spinlock::new(unsafe { SerialPort::new(0x3F8) });

pub fn init() {
    SERIAL1.lock().init();
}

pub struct SerialLogger;
pub static SERIAL_LOGGER: SerialLogger = SerialLogger;

impl log::Log for SerialLogger {
    fn enabled(&self, _m: &log::Metadata) -> bool { true }

    fn log(&self, record: &log::Record) {
        use core::fmt::Write;
        let _ = writeln!(
            SERIAL1.lock(),
            "[{:5}] {}",
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}
