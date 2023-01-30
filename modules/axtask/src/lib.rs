#![cfg_attr(not(test), no_std)]
#![feature(ptr_internals)]
#![feature(const_trait_impl)]

#[macro_use]
extern crate log;
extern crate alloc;

mod run_queue;
mod task;
mod tests;

use alloc::sync::Arc;
use core::ops::DerefMut;
use lazy_init::LazyInit;
use run_queue::{AxRunQueue, RUN_QUEUE};

pub use task::AxTask;

cfg_if::cfg_if! {
    if #[cfg(feature = "sched_fifo")] {
        use scheduler::FifoSchedState as SchedStateImpl;
        use scheduler::FifoScheduler as SchedulerImpl;
    } else if #[cfg(feature = "sched_rr")] {
        use scheduler::RRScheduler as SchedulerImpl;
        use scheduler::RRSchedState as SchedStateImpl;
    }
}

// TODO: per-CPU
pub(crate) static mut CURRENT_TASK: LazyInit<Arc<AxTask>> = LazyInit::new();

pub(crate) fn set_current(task: Arc<AxTask>) {
    assert!(!axhal::arch::irqs_enabled());
    let old_task = core::mem::replace(unsafe { CURRENT_TASK.deref_mut() }, task);
    drop(old_task)
}

pub fn current<'a>() -> &'a Arc<AxTask> {
    unsafe { &CURRENT_TASK }
}

pub fn init_scheduler() {
    let rq = AxRunQueue::new();
    unsafe { CURRENT_TASK.init_by(rq.init_task().clone()) };
    RUN_QUEUE.init_by(spin::Mutex::new(rq));
}

pub fn spawn<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    let task = AxTask::new(f, "");
    RUN_QUEUE.lock().add_task(task);
}

pub fn yield_now() {
    RUN_QUEUE.lock().yield_current();
}

pub fn exit(exit_code: i32) -> ! {
    RUN_QUEUE.lock().exit_current(exit_code)
}

pub(crate) fn resched() {
    RUN_QUEUE.lock().resched();
}
