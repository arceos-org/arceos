#[cfg(feature = "multitask")]
pub use axtask::{current, sleep, sleep_until, spawn, yield_now, TaskId};

#[cfg(not(feature = "multitask"))]
pub fn yield_now() {
    core::hint::spin_loop()
}

#[cfg(not(feature = "multitask"))]
pub fn exit(exit_code: i32) -> ! {
    axlog::debug!("main task exited: exit_code={}", exit_code);
    axhal::misc::terminate()
}
