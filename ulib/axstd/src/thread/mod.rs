//! Native threads.

#[cfg(feature = "multitask")]
mod multi;
#[cfg(feature = "multitask")]
pub use multi::*;

use arceos_api::task as api;
use core::num::NonZero;

/// Current thread gives up the CPU time voluntarily, and switches to another
/// ready thread.
///
/// For single-threaded configuration (`multitask` feature is disabled), we just
/// relax the CPU and wait for incoming interrupts.
pub fn yield_now() {
    api::ax_yield_now();
}

/// Exits the current thread.
///
/// For single-threaded configuration (`multitask` feature is disabled),
/// it directly terminates the main thread and shutdown.
pub fn exit(exit_code: i32) -> ! {
    api::ax_exit(exit_code);
}

/// Current thread is going to sleep for the given duration.
///
/// If one of `multitask` or `irq` features is not enabled, it uses busy-wait
/// instead.
pub fn sleep(dur: core::time::Duration) {
    sleep_until(arceos_api::time::ax_wall_time() + dur);
}

/// Current thread is going to sleep, it will be woken up at the given deadline.
///
/// If one of `multitask` or `irq` features is not enabled, it uses busy-wait
/// instead.
pub fn sleep_until(deadline: arceos_api::time::AxTimeValue) {
    api::ax_sleep_until(deadline);
}

/// Returns an estimate of the default amount of parallelism a program should use.
///
/// Here we directly return the number of available logical CPUs, representing
/// the theoretical maximum parallelism.
pub fn available_parallelism() -> crate::io::Result<NonZero<usize>> {
    NonZero::new(arceos_api::modules::axconfig::plat::CPU_NUM)
        .ok_or_else(|| panic!("No available CPUs found, cannot determine parallelism"))
}
