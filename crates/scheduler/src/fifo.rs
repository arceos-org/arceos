use alloc::{collections::VecDeque, sync::Arc};

use crate::{BaseScheduler, Schedulable};

#[derive(Default)]
pub struct FifoSchedState;

pub struct FifoScheduler<T: Schedulable<FifoSchedState>> {
    ready_queue: VecDeque<Arc<T>>,
}

impl<T: Schedulable<FifoSchedState>> FifoScheduler<T> {
    pub const fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
}

impl<T: Schedulable<FifoSchedState>> BaseScheduler<FifoSchedState, T> for FifoScheduler<T> {
    fn init(&mut self) {}

    fn add_task(&mut self, task: Arc<T>) {
        self.ready_queue.push_back(task);
    }

    fn remove_task(&mut self, task: &Arc<T>) {
        // TODO: more efficient
        self.ready_queue.retain(|t| !Arc::ptr_eq(t, task));
    }

    fn yield_task(&mut self, task: Arc<T>) {
        self.remove_task(&task);
        self.add_task(task);
    }

    fn pick_next_task(&mut self, _prev: &Arc<T>) -> Option<&Arc<T>> {
        self.ready_queue.front()
    }

    fn task_tick(&mut self, _current: &Arc<T>) -> bool {
        false // no reschedule
    }
}
