//! PL011 UART.

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

use memory_addr::{PhysAddr, VirtAddr};
use spin::Mutex;

const UART_BASE: PhysAddr = PhysAddr::from(0x0900_0000);
#[allow(unused)]
const UART_IRQ_NUM: usize = 33;

static UART: Mutex<Pl011Uart> = Mutex::new(Pl011Uart::new(VirtAddr::from(
    UART_BASE.as_usize() + axconfig::PHYS_VIRT_OFFSET,
)));

register_structs! {
    Pl011UartRegs {
        /// Data Register.
        (0x00 => dr: ReadWrite<u32>),
        (0x04 => _reserved0),
        /// Flag Register.
        (0x18 => fr: ReadOnly<u32>),
        (0x1c => _reserved1),
        /// Control register.
        (0x30 => cr: ReadWrite<u32>),
        /// Interrupt FIFO Level Select Register.
        (0x34 => ifls: ReadWrite<u32>),
        /// Interrupt Mask Set Clear Register.
        (0x38 => imsc: ReadWrite<u32>),
        /// Raw Interrupt Status Register.
        (0x3c => ris: ReadOnly<u32>),
        /// Masked Interrupt Status Register.
        (0x40 => mis: ReadOnly<u32>),
        /// Interrupt Clear Register.
        (0x44 => icr: WriteOnly<u32>),
        (0x48 => @END),
    }
}

struct Pl011Uart {
    base_vaddr: VirtAddr,
}

impl Pl011Uart {
    const fn new(base_vaddr: VirtAddr) -> Self {
        Self { base_vaddr }
    }

    const fn regs(&self) -> &Pl011UartRegs {
        unsafe { &*(self.base_vaddr.as_ptr() as *const _) }
    }

    fn init(&mut self) {
        // clear all irqs
        self.regs().icr.set(0x3ff);

        // set fifo trigger level
        self.regs().ifls.set(0); // 1/8 rxfifo, 1/8 txfifo.

        // enable rx interrupt
        self.regs().imsc.set(1 << 4); // rxim

        // enable receive
        self.regs().cr.set((1 << 0) | (1 << 8) | (1 << 9)); // tx enable, rx enable, uart enable
    }

    fn putchar(&mut self, c: u8) {
        while self.regs().fr.get() & (1 << 5) != 0 {}
        self.regs().dr.set(c as u32);
    }

    fn getchar(&mut self) -> Option<u8> {
        if self.regs().fr.get() & (1 << 4) == 0 {
            Some(self.regs().dr.get() as u8)
        } else {
            None
        }
    }
}

pub fn console_putchar(c: u8) {
    UART.lock().putchar(c);
}

pub fn console_getchar() -> Option<u8> {
    UART.lock().getchar()
}

pub fn init() {
    UART.lock().init();
}
