//! Interrupt management.

use axcpu::trap::{IRQ, register_trap_handler};

pub use axplat::irq::{handle, register, set_enable, unregister, IPI_IRQ_NUM};

#[cfg(feature = "ipi")]
pub use crate::platform::irq::{IPI_IRQ_NUM, send_ipi_all_others, send_ipi_one};

#[register_trap_handler(IRQ)]
fn irq_handler(vector: usize) -> bool {
    let guard = kernel_guard::NoPreempt::new();
    handle(vector);
    drop(guard); // rescheduling may occur when preemption is re-enabled.
    true
}
