use alloc::collections::BTreeMap;

use kernel_guard::NoPreemptIrqSave;
use kspin::SpinNoIrq;

use crate::{AxTaskRef, current_run_queue, select_run_queue};

/// Task registry by task id
static TASK_REGISTRY: SpinNoIrq<BTreeMap<u64, AxTaskRef>> = SpinNoIrq::new(BTreeMap::new());

/// Register a task to the task registry.
/// 
/// Should be called in the process of task spawn.
pub fn register_task(task: AxTaskRef) {
    let mut tasks = TASK_REGISTRY.lock();
    let id = task.id().as_u64();
    tasks.insert(id, task);
    debug!("Task {} registered", id);
}

/// Unregister a task from the task registry.
/// 
/// Should be called in the process of task exit.
pub fn unregister_task(id: u64) {
    let mut tasks = TASK_REGISTRY.lock();
    tasks.remove(&id);
    debug!("Task {} unregistered", id);
}

/// Find a task by its ID.
pub fn find_task_by_id(id: u64) -> Option<AxTaskRef> {
    let tasks = TASK_REGISTRY.lock();
    tasks.get(&id).cloned()
}

/// Unpark a task by its id.
pub fn unpark_task(id: u64, resched: bool) {
    if let Some(task) = find_task_by_id(id) {
        select_run_queue::<NoPreemptIrqSave>(&task).unpark_task(task, resched);
    } 
}

/// Park the current task.
pub fn park_current_task() {
    let mut cur_rq = current_run_queue::<NoPreemptIrqSave>();
    cur_rq.park_current_task();
}
