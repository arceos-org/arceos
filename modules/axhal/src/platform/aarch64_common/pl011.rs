//! PL011 UART.

use arm_pl011::pl011::Pl011Uart;
use memory_addr::PhysAddr;
use spinlock::SpinNoIrq;

use crate::mem::phys_to_virt;

const UART_BASE: PhysAddr = PhysAddr::from(axconfig::UART_PADDR);

static UART: SpinNoIrq<Pl011Uart> =
    SpinNoIrq::new(Pl011Uart::new(phys_to_virt(UART_BASE).as_mut_ptr()));

/// Writes a byte to the console.
pub fn putchar(c: u8) {
    let mut uart = UART.lock();
    match c {
        b'\n' => {
            uart.putchar(b'\r');
            uart.putchar(b'\n');
        }
        c => uart.putchar(c),
    }
}

/// Reads a byte from the console, or returns [`None`] if no input is available.
pub fn getchar() -> Option<u8> {
    UART.lock().getchar()
}

/// Initialize the UART
pub fn init_early() {
    UART.lock().init();
}

/// Set UART IRQ Enable
pub fn init() {
    #[cfg(feature = "irq")]
    crate::platform::irq::set_enable(axconfig::UART_IRQ_NUM, true);
}

/// UART IRQ Handler
pub fn handle() {
    let is_receive_interrupt = UART.lock().is_receive_interrupt();
    UART.lock().ack_interrupts();
    if is_receive_interrupt {
        while let Some(c) = getchar() {
            putchar(c);
        }
    }
}
