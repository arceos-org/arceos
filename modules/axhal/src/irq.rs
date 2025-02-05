//! Interrupt management.

use axcpu::trap::{IRQ, register_trap_handler};

pub use axplat::irq::{handle, register, set_enable, unregister};

#[register_trap_handler(IRQ)]
fn irq_handler(vector: usize) -> bool {
    let guard = kernel_guard::NoPreempt::new();
    handle(vector);
    drop(guard); // rescheduling may occur when preemption is re-enabled.
    true
}
