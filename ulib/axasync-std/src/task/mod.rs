use arceos_api::task as api;
pub use core::task::*;

mod multi;
pub use multi::*;

/// Current coroutine gives up the CPU time voluntarily, and switches to another
/// ready task.
///
/// For single-threaded configuration (`multitask` feature is disabled), we just
/// relax the CPU and wait for incoming interrupts.
pub async fn yield_now() {
    api::ax_yield_now_f().await;
}

/// Exits the current coroutine.
///
/// For single-threaded configuration (`multitask` feature is disabled),
/// it directly terminates the main thread and shutdown.
pub async fn exit(exit_code: i32) {
    api::ax_exit_f(exit_code).await;
}

/// Current coroutine is going to sleep for the given duration.
///
/// If one of `multitask` or `irq` features is not enabled, it uses busy-wait
/// instead.
pub async fn sleep(dur: core::time::Duration) {
    sleep_until(arceos_api::time::ax_wall_time() + dur).await;
}

/// Current thread is going to sleep, it will be woken up at the given deadline.
///
/// If one of `multitask` or `irq` features is not enabled, it uses busy-wait
/// instead.
pub async fn sleep_until(deadline: arceos_api::time::AxTimeValue) {
    api::ax_sleep_until_f(deadline).await;
}
