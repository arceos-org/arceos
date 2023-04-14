use crate::run_queue::RUN_QUEUE;
use crate::task::{CurrentTask, TaskInner};

#[doc(cfg(feature = "multitask"))]
pub use crate::task::TaskId;
#[doc(cfg(feature = "multitask"))]
pub use crate::wait_queue::WaitQueue;

pub fn current_may_uninit() -> Option<CurrentTask> {
    CurrentTask::try_get()
}

pub fn current() -> CurrentTask {
    CurrentTask::get()
}

pub fn init_scheduler() {
    info!("Initialize scheduling...");

    crate::run_queue::init();
    crate::timers::init();

    if cfg!(feature = "sched_fifo") {
        info!("  use FIFO scheduler.");
    } else if cfg!(feature = "sched_rr") {
        info!("  use Round-robin scheduler.");
    }
}

pub fn init_scheduler_secondary() {
    crate::run_queue::init_secondary();
}

/// Handle periodic timer ticks for task manager, e.g. advance scheduler, update timer.
pub fn on_timer_tick() {
    crate::timers::check_events();
    RUN_QUEUE.lock().scheduler_timer_tick();
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

pub fn sleep(dur: core::time::Duration) {
    let deadline = axhal::time::current_time() + dur;
    RUN_QUEUE.lock().sleep_until(deadline);
}

pub fn sleep_until(deadline: axhal::time::TimeValue) {
    RUN_QUEUE.lock().sleep_until(deadline);
}

pub fn exit(exit_code: i32) -> ! {
    RUN_QUEUE.lock().exit_current(exit_code)
}
