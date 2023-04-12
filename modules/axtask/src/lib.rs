#![cfg_attr(not(test), no_std)]
#![feature(const_trait_impl)]
#![feature(doc_auto_cfg)]
#![feature(doc_cfg)]

#[macro_use]
extern crate log;

struct KernelGuardIfImpl;

#[crate_interface::impl_interface]
impl kernel_guard::KernelGuardIf for KernelGuardIfImpl {
    fn disable_preempt() {
        #[cfg(all(feature = "multitask", feature = "preempt"))]
        if let Some(curr) = current_may_uninit() {
            curr.disable_preempt();
        }
    }

    fn enable_preempt() {
        #[cfg(all(feature = "multitask", feature = "preempt"))]
        if let Some(curr) = current_may_uninit() {
            curr.enable_preempt(true);
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg_attr(not(feature = "multitask"), path = "api_s.rs")]
mod api;

#[doc(cfg(feature = "multitask"))]
pub use self::api::*;
pub use self::api::{exit, sleep, sleep_until, yield_now};

cfg_if::cfg_if! {
if #[cfg(feature = "multitask")] {

extern crate alloc;

mod run_queue;
mod task;
mod timers;
mod wait_queue;

use self::run_queue::{AxRunQueue, RUN_QUEUE};
use self::task::{CurrentTask, TaskInner};

type AxTaskRef = alloc::sync::Arc<AxTask>;

cfg_if::cfg_if! {
    if #[cfg(feature = "sched_fifo")] {
        type AxTask = scheduler::FifoTask<TaskInner>;
        type Scheduler = scheduler::FifoScheduler<TaskInner>;
    } else if #[cfg(feature = "sched_rr")] {
        const MAX_TIME_SLICE: usize = 5;
        type AxTask = scheduler::RRTask<TaskInner, MAX_TIME_SLICE>;
        type Scheduler = scheduler::RRScheduler<TaskInner, MAX_TIME_SLICE>;
    }
}

}
} // cfg_if::cfg_if!

pub fn run_idle() -> ! {
    loop {
        #[cfg(feature = "multitask")]
        yield_now();
        debug!("idle task: waiting for IRQs...");
        axhal::arch::wait_for_irqs();
    }
}
