//! Interrupt management.

use axcpu::trap::{IRQ, register_trap_handler};

pub use axplat::irq::{handle, register, set_enable, unregister};

#[cfg(feature = "ipi")]
pub use axplat::irq::{IpiTarget, send_ipi};

#[cfg(feature = "ipi")]
pub use axconfig::devices::IPI_IRQ;

/// The generic IRQ (Interrupt Request) handler.
///
/// This function is registered as the IRQ trap handler and is called
/// whenever an interrupt occurs. It disables kernel preemption during
/// the handling to ensure atomicity, then re-enables preemption after
/// handling, allowing the scheduler to reschedule tasks if needed.
#[register_trap_handler(IRQ)]
pub fn irq_handler(vector: usize) -> bool {
    let guard = kernel_guard::NoPreempt::new();
    handle(vector);
    drop(guard); // rescheduling may occur when preemption is re-enabled.
    true
}
