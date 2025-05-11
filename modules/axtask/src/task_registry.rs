use alloc::collections::BTreeMap;

use kernel_guard::{BaseGuard, NoPreemptIrqSave};
use kspin::SpinNoIrq;

use crate::{current, current_run_queue, run_queue, select_run_queue, AxTaskRef};

static TASK_REGISTRY: SpinNoIrq<BTreeMap<u64,AxTaskRef>> = SpinNoIrq::new(BTreeMap::new());

pub fn register_task(task: AxTaskRef) {
	let mut tasks = TASK_REGISTRY.lock();
	let id = task.id().as_u64();
	tasks.insert(id, task);
	debug!("Task {} registered", id);
}

pub fn unregister_task(id:u64) {
	let mut tasks = TASK_REGISTRY.lock();
	tasks.remove(&id);
	debug!("Task {} registered", id);
}

pub fn find_task_by_id(id: u64) -> Option<AxTaskRef> {
	let tasks = TASK_REGISTRY.lock();
	tasks.get(&id).cloned()
}

pub fn unpark_task(id:u64) {
	if let Some(task) = find_task_by_id(id) {
		select_run_queue::<NoPreemptIrqSave>(&task).unblock_task(task, true);
	} else {
		debug!("Task {} not found", id);
	}
}

pub fn park_current_task() {
	let mut cur_rq = current_run_queue::<NoPreemptIrqSave>();
	cur_rq.park_current_task();
}

