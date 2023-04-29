//! Native threads.

#[cfg(feature = "multitask")]
pub use axtask::{current, set_priority, spawn, TaskId};

/// Current task gives up the CPU time voluntarily, and switches to another
/// ready task.
///
/// For single-task situation (`multitask` feature is disabled), we just relax
/// the CPU and wait for incoming interrupts.
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

/// Exits the current task.
///
/// For single-task situation (`multitask` feature is disabled), directly
/// terminate the main task and shutdown.
pub fn exit(exit_code: i32) -> ! {
    #[cfg(feature = "multitask")]
    axtask::exit(exit_code);
    #[cfg(not(feature = "multitask"))]
    {
        axlog::debug!("main task exited: exit_code={}", exit_code);
        axhal::misc::terminate()
    }
}

/// Current task is going to sleep for the given duration.
///
/// If one of `multitask` or `irq` features is not enabled, it uses busy-wait
/// instead.
pub fn sleep(dur: core::time::Duration) {
    sleep_until(axhal::time::current_time() + dur);
}

/// Current task is going to sleep, it will be woken up at the given deadline.
///
/// If one of `multitask` or `irq` features is not enabled, it uses busy-wait
/// instead.
pub fn sleep_until(deadline: axhal::time::TimeValue) {
    #[cfg(feature = "multitask")]
    axtask::sleep_until(deadline);
    #[cfg(not(feature = "multitask"))]
    axhal::time::busy_wait_until(deadline);
}
