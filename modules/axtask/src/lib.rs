#![cfg_attr(not(test), no_std)]
#![feature(const_trait_impl)]

cfg_if::cfg_if! {
if #[cfg(feature = "multitask")] {

#[macro_use]
extern crate log;
extern crate alloc;

mod run_queue;
mod task;

#[cfg(test)]
mod tests;

use alloc::sync::Arc;
use core::ops::DerefMut;
use lazy_init::LazyInit;
use run_queue::{AxRunQueue, RUN_QUEUE};
use task::TaskInner;

pub use task::TaskId;

cfg_if::cfg_if! {
    if #[cfg(feature = "sched_fifo")] {
        type AxTask = scheduler::FifoTask<TaskInner>;
        type Scheduler = scheduler::FifoScheduler<TaskInner>;
    } else if #[cfg(feature = "sched_rr")] {
        type AxTask = scheduler::RRTask<TaskInner>;
        type Scheduler = scheduler::RRScheduler<TaskInner>;
    }
}

type AxTaskRef = Arc<AxTask>;

// TODO: per-CPU
pub(crate) static mut CURRENT_TASK: LazyInit<AxTaskRef> = LazyInit::new();

pub(crate) fn set_current(task: AxTaskRef) {
    assert!(!axhal::arch::irqs_enabled());
    let old_task = core::mem::replace(unsafe { CURRENT_TASK.deref_mut() }, task);
    drop(old_task)
}

pub fn current<'a>() -> &'a AxTaskRef {
    unsafe { &CURRENT_TASK }
}

pub fn init_scheduler() {
    let rq = AxRunQueue::new();
    unsafe { CURRENT_TASK.init_by(rq.init_task().clone()) };
    RUN_QUEUE.init_by(spin::Mutex::new(rq));
    if cfg!(feature = "sched_fifo") {
        info!("  use FIFO scheduler.");
    } else if cfg!(feature = "sched_rr") {
        info!("  use Round-robin scheduler.");
    }
}

pub fn spawn<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    let task = TaskInner::new(f, "");
    RUN_QUEUE.lock().add_task(task);
}

pub fn yield_now() {
    RUN_QUEUE.lock().yield_current();
}

pub fn exit(exit_code: i32) -> ! {
    RUN_QUEUE.lock().exit_current(exit_code)
}

} else { // if #[cfg(feature = "multitask")]

pub fn yield_now() {}

} // else
} // cfg_if::cfg_if!
