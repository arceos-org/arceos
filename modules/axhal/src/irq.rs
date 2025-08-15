//! Interrupt management.

use core::task::Waker;

use axcpu::trap::{IRQ, register_trap_handler};
use axio::PollSet;
pub use axplat::irq::{handle, register, set_enable, unregister};

static POLL_TABLE: [PollSet; 0x30] = [const { PollSet::new() }; 0x30];
fn poll_handler(irq: usize) {
    POLL_TABLE[irq].wake();
}

/// Registers a waker for a IRQ interrupt.
pub fn register_irq_waker(irq: u32, waker: &Waker) {
    POLL_TABLE[irq as usize].register(waker);
    axplat::irq::register(irq as usize, poll_handler);
}

#[register_trap_handler(IRQ)]
fn irq_handler(vector: usize) -> bool {
    let guard = kernel_guard::NoPreempt::new();
    handle(vector);
    drop(guard); // rescheduling may occur when preemption is re-enabled.
    true
}
