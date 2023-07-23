//! Native threads.

#[cfg(feature = "multitask")]
mod multi;
#[cfg(feature = "multitask")]
pub use multi::*;

/// Current thread gives up the CPU time voluntarily, and switches to another
/// ready thread.
///
/// For single-threaded configuration (`multitask` feature is disabled), we just
/// relax the CPU and wait for incoming interrupts.
pub fn yield_now() {
    #[cfg(feature = "multitask")]
    axtask::yield_now();
    #[cfg(not(feature = "multitask"))]
    if cfg!(feature = "irq") {
        axhal::arch::wait_for_irqs();
    } else {
        core::hint::spin_loop();
    }
}

/// Exits the current thread.
///
/// For single-threaded configuration (`multitask` feature is disabled),
/// it directly terminates the main thread and shutdown.
pub fn exit(_exit_code: i32) -> ! {
    #[cfg(feature = "multitask")]
    axtask::exit(_exit_code);
    #[cfg(not(feature = "multitask"))]
    axhal::misc::terminate();
}

/// Current thread is going to sleep for the given duration.
///
/// If one of `multitask` or `irq` features is not enabled, it uses busy-wait
/// instead.
pub fn sleep(dur: core::time::Duration) {
    sleep_until(axhal::time::current_time() + dur);
}

/// Current thread is going to sleep, it will be woken up at the given deadline.
///
/// If one of `multitask` or `irq` features is not enabled, it uses busy-wait
/// instead.
pub fn sleep_until(deadline: axhal::time::TimeValue) {
    #[cfg(feature = "multitask")]
    axtask::sleep_until(deadline);
    #[cfg(not(feature = "multitask"))]
    axhal::time::busy_wait_until(deadline);
}
