//! RISC-V PLIC implementation.
#![no_std]
#![deny(missing_docs)]
#![feature(const_nonnull_new, const_option, const_ptr_as_ref)]

use core::mem::size_of;
use core::ptr::NonNull;

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::ReadWrite;

#[macro_use]
extern crate log;

#[repr(C, align(4096))]
struct ContextLocal {
    priority_threshold: ReadWrite<u32>,
    claim_or_completion: ReadWrite<u32>,
    _reserved: [u8; 4096 - 2 * size_of::<u32>()],
}

register_structs! {
    #[allow(non_snake_case)]
    PlicRegs{
        /// Interrupt Source Priority Registers.
        (0x0000 => PRIORITY: [ReadWrite<u32>; 1024]),
        /// Interrupt Pending Registers.
        (0x1000 => PENDING: [ReadWrite<u32>; 32]),
        (0x1080 => _reserved_1),
        /// Interrupt Enable Registers.
        (0x2000 => ENABLE: [ReadWrite<u32>; 1024 * 15872 / 32]),
        (0x1f_2000 => _reserved_2),
        /// Interrupt Threshold/Claim Registers.
        (0x20_0000 => CONTEXT: [ContextLocal; 15872]),
        (0x400_0000 => @END),
    }

}

/// RISC-V PLIC struct.
pub struct Plic {
    base: NonNull<PlicRegs>,
}

impl Plic {
    /// Create a new instance.
    pub const fn new(base: *mut u8) -> Self {
        Plic {
            base: NonNull::new(base as *mut PlicRegs).unwrap().cast(),
        }
    }

    const fn regs(&self) -> &PlicRegs {
        unsafe { self.base.as_ref() }
    }

    /// Set interrupt priority by `irq`.
    pub fn set_priority(&self, irq: usize, priority: u32) {
        assert!(priority < 8);
        self.regs().PRIORITY[irq].set(priority);
        info!(
            "PLIC set_priority@addr: {:#x}, irq: {}, priority: {}",
            &self.regs().PRIORITY[irq] as *const _ as usize,
            irq,
            priority
        );
    }

    /// Get interrupt priority by `irq`.
    pub fn get_priority(&self, irq: usize) -> u32 {
        self.regs().PRIORITY[irq].get()
    }

    /// Set threshold.
    pub fn set_threshold(&self, hart_id: usize, priority: usize, threshold: u32) {
        let id = hart_id * 2 + priority;
        self.regs().CONTEXT[id].priority_threshold.set(threshold);
    }

    /// Get threshold.
    pub fn get_threshold(&self, hart_id: usize, priority: usize) -> u32 {
        let id = hart_id * 2 + priority;
        self.regs().CONTEXT[id].priority_threshold.get()
    }

    /// Enable the interrupt for the given hart.
    pub fn enable(&self, hart_id: usize, irq_num: usize) {
        let context_base = 0x80 * (hart_id * 2 + 1);
        info!("context_base: {:#x}", context_base);
        let (reg_id, reg_shift) = (irq_num / 32, irq_num % 32);
        let pos = context_base / size_of::<u32>() + reg_id;
        self.regs().ENABLE[pos].set(self.regs().ENABLE[pos].get() | (1 << reg_shift));
    }

    /// Disable interrupt source for the given hart.
    pub fn disable(&self, hart_id: usize, irq_num: usize) {
        let context_base = 0x80 * (hart_id * 2 + 1);
        let (reg_id, reg_shift) = (irq_num / 32, irq_num % 32);
        let pos = context_base / size_of::<u32>() + reg_id;
        self.regs().ENABLE[pos].set(self.regs().ENABLE[pos].get() & !(1 << reg_shift));
    }

    /// Claim a interrupt for the given hart, return the interrupt number.
    pub fn claim(&self, hart_id: usize) -> u32 {
        let ctx = hart_id * 2 + 1;
        self.regs().CONTEXT[ctx].claim_or_completion.get()
    }

    /// Mark a interrupt is completed.
    pub fn complete(&self, hart_id: usize, completion: u32) {
        let ctx = hart_id * 2 + 1;
        self.regs().CONTEXT[ctx].claim_or_completion.set(completion);
    }
}

unsafe impl Sync for Plic {}
unsafe impl Send for Plic {}
