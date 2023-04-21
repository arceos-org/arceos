//! Native threads.

#[cfg(feature = "multitask")]
pub use axtask::{current, exit, sleep, sleep_until, spawn, yield_now, TaskId};

/// For single-task situation, we just relax the CPU and wait for incoming
/// interrupts.
#[cfg(not(feature = "multitask"))]
pub fn yield_now() {
    axhal::arch::wait_for_irqs();
}

/// For single-task situation, situation, directly terminate the main task and
/// shutdown.
#[cfg(not(feature = "multitask"))]
pub fn exit(exit_code: i32) -> ! {
    axlog::debug!("main task exited: exit_code={}", exit_code);
    axhal::misc::terminate()
}
