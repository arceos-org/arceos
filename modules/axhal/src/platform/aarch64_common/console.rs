//! Console management.

use crate::mem::phys_to_virt;
use memory_addr::PhysAddr;
use spinlock::SpinNoIrq;
use dw_apb_uart::DW8250;
use arm_pl011::pl011::Pl011Uart;
use driver_common::UartDriver;

const UART_BASE: PhysAddr = PhysAddr::from(axconfig::UART_PADDR);

static DRIVER_8250: DW8250 = DW8250::new(phys_to_virt(UART_BASE).as_usize()); 
static DRIVER_PL011: Pl011Uart = Pl011Uart::new(phys_to_virt(UART_BASE).as_mut_ptr());

lazy_static::lazy_static! {
    static ref MACHINE: &'static str = of::machin_name();
    static ref UART: SpinNoIrq<&'static dyn UartDriver> = {
        if MACHINE.contains("BST") {
            SpinNoIrq::new(&DRIVER_8250)
        } else {
            SpinNoIrq::new(&DRIVER_PL011)
        }
    };
}

/// Feature use bootargs earlycon serial,now still hard coding
pub(crate) fn console_early_init() {
    unsafe {crate::platform::aarch64_common::mem::idmap_device(UART_BASE.as_usize());}
    UART.lock().init();
}

/// Writes a byte to the console.
pub fn putchar(c: u8) {
    let uart = UART.lock();
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

/// Set UART IRQ Enable
#[cfg(feature = "irq")]
fn init_8250_irq() {
    UART.lock().set_ier(true);
    crate::irq::register_handler(crate::platform::irq::UART_IRQ_NUM, dw_apb_uart_handle);
}

/// Set UART IRQ Enable
fn init_pl011_irq() {
    #[cfg(feature = "irq")]
    crate::irq::set_enable(crate::platform::irq::UART_IRQ_NUM, true);
}

/// Set UART IRQ Enable
#[cfg(feature = "irq")]
pub fn init_irq() {
    if MACHINE.contains("BST") {
        init_8250_irq();
    } else {
       init_pl011_irq();
    }
}

/// UART IRQ Handler
fn dw_apb_uart_handle() {
    trace!("Uart IRQ Handler");
}

/// UART IRQ Handler
pub fn pl011_handle() {
    let is_receive_interrupt = UART.lock().is_receive_interrupt();
    UART.lock().ack_interrupts();
    if is_receive_interrupt {
        while let Some(c) = getchar() {
            putchar(c);
        }
    }
}
