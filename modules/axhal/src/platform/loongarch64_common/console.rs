//! Uart 16550.

use core::fmt::Write;
use memory_addr::PhysAddr;
use spinlock::SpinNoIrq;

const UART_BASE: PhysAddr = PhysAddr::from(axconfig::UART_BASE_ADDR);
const UART_DAT_OFFSET: usize = 0x0;
const UART_LSR_OFFSET: usize = 0x5;
const UART_LSR_TFE_OFFSET: usize = 0x5;
const UART_LSR_DR_OFFSET: usize = 0x0;

static COM1: SpinNoIrq<Uart> = SpinNoIrq::new(Uart::new(UART_BASE.as_usize()));

pub struct Uart {
    base_address: usize,
}

impl Uart {
    pub const fn new(base_address: usize) -> Self {
        Uart { base_address }
    }

    pub fn putchar(&mut self, c: u8) {
        let ptr = self.base_address as *mut u8;
        loop {
            unsafe {
                let c = ptr.add(UART_LSR_OFFSET).read_volatile();
                if c & (1 << UART_LSR_TFE_OFFSET) != 0 {
                    break;
                }
            }
        }
        unsafe {
            ptr.add(UART_DAT_OFFSET).write_volatile(c);
        }
    }

    pub fn getchar(&mut self) -> Option<u8> {
        let ptr = self.base_address as *mut u8;
        unsafe {
            if ptr.add(UART_LSR_OFFSET).read_volatile() & (1 << UART_LSR_DR_OFFSET) == 0 {
                // The DR bit is 0, meaning no data
                None
            } else {
                // The DR bit is 1, meaning data!
                Some(ptr.add(UART_DAT_OFFSET).read_volatile())
            }
        }
    }
}
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            self.putchar(c);
        }
        Ok(())
    }
}

/// Writes a byte to the console.
pub fn putchar(c: u8) {
    let mut uart = COM1.lock();
    match c {
        b'\n' => {
            uart.putchar(b'\r');
            uart.putchar(b'\n');
        }
        c => uart.putchar(c),
    }
}

pub fn write_fmt(args: core::fmt::Arguments) {
    use core::fmt::Write;
    COM1.lock().write_fmt(args).unwrap();
}

/// Reads a byte from the console, or returns [`None`] if no input is available.
pub fn getchar() -> Option<u8> {
    COM1.lock().getchar()
}
