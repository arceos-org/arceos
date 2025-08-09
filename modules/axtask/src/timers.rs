use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{
    cmp::Reverse,
    hash::{Hash, Hasher},
};

use axhal::time::{TimeValue, wall_time};
use foldhash::fast::RandomState;
use kernel_guard::{NoOp, NoPreemptIrqSave};
use kspin::{SpinNoIrq, SpinRaw};
use priority_queue::PriorityQueue;
use spin::Lazy;

use crate::{AxTaskRef, WeakAxTaskRef, select_run_queue};

static TIMER_CALLBACKS: SpinRaw<Vec<Box<dyn Fn(TimeValue) + Send + Sync>>> =
    SpinRaw::new(Vec::new());

pub fn register_timer_callback<F>(callback: F)
where
    F: Fn(TimeValue) + Send + Sync + 'static,
{
    let _g = NoPreemptIrqSave::new();
    TIMER_CALLBACKS.lock().push(Box::new(callback));
}

struct TaskPtr(WeakAxTaskRef);

impl TaskPtr {
    fn new(task: &AxTaskRef) -> Self {
        TaskPtr(Arc::downgrade(task))
    }

    fn upgrade(&self) -> Option<AxTaskRef> {
        self.0.upgrade()
    }
}

impl PartialEq for TaskPtr {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}

impl Eq for TaskPtr {}

impl Hash for TaskPtr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

static TIMER_LIST: Lazy<SpinNoIrq<PriorityQueue<TaskPtr, Reverse<TimeValue>, RandomState>>> =
    Lazy::new(|| SpinNoIrq::new(PriorityQueue::with_default_hasher()));

pub fn set_alarm_wakeup(deadline: TimeValue, task: &AxTaskRef) {
    TIMER_LIST
        .lock()
        .push(TaskPtr::new(task), Reverse(deadline));
}

pub fn clear_alarm_wakeup(task: &AxTaskRef) {
    TIMER_LIST.lock().remove(&TaskPtr::new(task));
}

pub fn check_events() {
    let now = wall_time();
    for callback in TIMER_CALLBACKS.lock().iter() {
        callback(now);
    }
    let mut timer_list = TIMER_LIST.lock();
    while let Some((task_ptr, _)) = timer_list.pop_if(|_, Reverse(deadline)| *deadline < now) {
        if let Some(task) = task_ptr.upgrade() {
            select_run_queue::<NoOp>(&task).unblock_task(task, true);
        }
    }
}
