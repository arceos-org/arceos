//! snps,dw-apb-uart serial driver

use crate::mem::phys_to_virt;
use dw_apb_uart::DW8250;
use memory_addr::PhysAddr;
use spinlock::SpinNoIrq;

const UART_BASE: PhysAddr = PhysAddr::from(axconfig::UART_PADDR);

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
pub fn getchar() -> Option<u8> {
    UART.lock().getchar()
}

/// UART simply initialize
pub fn init_early() {
    // SAFETY: idmap console mmio mem before paging
    unsafe {
        crate::platform::aarch64_common::mem::idmap_device(UART_BASE.as_usize());
    }
    //rk3588 uboot init uart, kernel not init uart
    //FEATURE: UART param init from dtb
    #[cfg(not(platform_family = "aarch64-rk3588j"))]
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
