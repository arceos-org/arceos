#![cfg_attr(not(test), no_std)]
#![feature(const_trait_impl)]

#[macro_use]
extern crate log;

cfg_if::cfg_if! {
if #[cfg(feature = "multitask")] {

extern crate alloc;

mod run_queue;
mod task;
mod wait_queue;

#[cfg(test)]
mod tests;

use alloc::sync::Arc;
use core::ops::DerefMut;
use lazy_init::LazyInit;

use self::run_queue::{AxRunQueue, RUN_QUEUE};
use self::task::TaskInner;

pub use self::task::TaskId;
pub use self::wait_queue::WaitQueue;

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

pub fn current_may_uninit<'a>() -> Option<&'a AxTaskRef> {
    unsafe { CURRENT_TASK.try_get() }
}

pub fn current<'a>() -> &'a AxTaskRef {
    unsafe { &CURRENT_TASK }
}

pub fn init_scheduler() {
    info!("Initialize scheduling...");

    let mut rq = AxRunQueue::new();
    unsafe { CURRENT_TASK.init_by(rq.get_mut().init_task().clone()) };
    RUN_QUEUE.init_by(rq);

    if cfg!(feature = "sched_fifo") {
        info!("  use FIFO scheduler.");
    } else if cfg!(feature = "sched_rr") {
        info!("  use Round-robin scheduler.");
    }
}

/// Handle periodic timer ticks for task manager, e.g. advance scheduler, update timer.
pub fn on_timer_tick() {
    RUN_QUEUE.lock().scheduler_timer_tick();
}

pub fn set_preemptiable(_enabled: bool) {
    #[cfg(feature = "preempt")]
    if let Some(curr) = current_may_uninit() {
        if _enabled {
            curr.enable_preempt(true);
        } else {
            curr.disable_preempt();
        }
    }
}

pub fn spawn<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    let task = TaskInner::new(f, "", axconfig::TASK_STACK_SIZE);
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

pub fn exit(exit_code: i32) -> ! {
    debug!("main task exited: exit_code={}", exit_code);
    axhal::misc::terminate()
}

} // else
} // cfg_if::cfg_if!
