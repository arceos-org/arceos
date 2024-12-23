//! snps,dw-apb-uart serial driver

use crate::mem::phys_to_virt;
use dw_apb_uart::DW8250;
use kspin::SpinNoIrq;
use memory_addr::PhysAddr;

const UART_BASE: PhysAddr = pa!(axconfig::devices::UART_PADDR);

static UART: SpinNoIrq<DW8250> = SpinNoIrq::new(DW8250::new(phys_to_virt(UART_BASE).as_usize()));

/// Writes a byte to the console.
pub fn putchar(c: u8) {
    let mut uart = UART.lock();
    match c {
        b'\r' | b'\n' => {
            uart.putchar(b'\r');
            uart.putchar(b'\n');
        }
        c => uart.putchar(c),
    }
}

/// Reads a byte from the console, or returns [`None`] if no input is available.
fn getchar() -> Option<u8> {
    UART.lock().getchar()
}

/// Write a slice of bytes to the console.
pub fn write_bytes(bytes: &[u8]) {
    for c in bytes {
        putchar(*c);
    }
}

/// Reads bytes from the console into the given mutable slice.
/// Returns the number of bytes read.
pub fn read_bytes(bytes: &mut [u8]) -> usize {
    let mut read_len = 0;
    while read_len < bytes.len() {
        if let Some(c) = getchar() {
            bytes[read_len] = c;
        } else {
            break;
        }
        read_len += 1;
    }
    read_len
}

/// UART simply initialize
pub fn init_early() {
    UART.lock().init();
}

/// Set UART IRQ Enable
#[cfg(feature = "irq")]
pub fn init_irq() {
    UART.lock().set_ier(true);
    crate::irq::register_handler(crate::platform::irq::UART_IRQ_NUM, handle);
}

/// UART IRQ Handler
pub fn handle() {
    trace!("Uart IRQ Handler");
}
