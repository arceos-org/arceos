//! Interrupt management.

use core::{array, task::Waker};

use axcpu::trap::{IRQ, register_trap_handler};
use axio::PollSet;
pub use axplat::irq::{handle, register, set_enable, unregister};
use lazyinit::LazyInit;

static POLL_TABLE: LazyInit<[PollSet; 0x30]> = LazyInit::new();
fn poll_handler(irq: usize) {
    POLL_TABLE[irq].wake();
}

/// Registers a waker for a IRQ interrupt.
pub fn register_irq_waker(irq: u32, waker: &Waker) {
    POLL_TABLE.call_once(|| array::from_fn(|_| PollSet::new()));
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
