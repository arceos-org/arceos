use core::ptr::NonNull;

use any_uart::*;
use kspin::SpinNoIrq;

use crate::mem::phys_to_virt;

static SEND: SpinNoIrq<Option<Sender>> = SpinNoIrq::new(None);
static RECV: SpinNoIrq<Option<Receiver>> = SpinNoIrq::new(None);

/// Initializes the console with dtb address and va offset func.
pub fn init(dtb: usize) -> Option<()> {
    let mut uart = any_uart::init(
        NonNull::new(phys_to_virt(dtb.into()).as_mut_ptr())?,
        |paddr| phys_to_virt(paddr.into()).as_mut_ptr(),
    )?;

    SEND.lock().replace(uart.tx.take().unwrap());
    RECV.lock().replace(uart.rx.take().unwrap());

    Some(())
}

/// Writes a byte to the console.
pub fn putchar(c: u8) {
    let mut g = SEND.lock();
    if let Some(uart) = g.as_mut() {
        match c {
            b'\n' => {
                let _ = block!(uart.write(b'\r'));
                let _ = block!(uart.write(b'\n'));
            }
            c => {
                let _ = block!(uart.write(c));
            }
        }
    }
}

/// Reads a byte from the console, or returns [`None`] if no input is available.
fn getchar() -> Option<u8> {
    let mut g = RECV.lock();
    let uart = g.as_mut()?;
    block!(uart.read()).ok()
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
