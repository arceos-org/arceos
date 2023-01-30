use alloc::{collections::VecDeque, sync::Arc};
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{BaseScheduler, Schedulable};

const MAX_TIME_SLICE: usize = 10;

#[derive(Default)]
pub struct RRSchedState {
    time_slice: AtomicUsize,
}

pub struct RRScheduler<T> {
    ready_queue: VecDeque<Arc<T>>,
}

impl<T: Schedulable<RRSchedState>> RRScheduler<T> {
    pub const fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
}

impl<T: Schedulable<RRSchedState>> BaseScheduler<RRSchedState, T> for RRScheduler<T> {
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

    fn task_tick(&mut self, current: &Arc<T>) -> bool {
        current.update_sched_state(|s| {
            let old_slice = s.time_slice.fetch_sub(1, Ordering::Release);
            if old_slice == 1 {
                s.time_slice.store(MAX_TIME_SLICE, Ordering::Release);
                self.yield_task(current.clone());
                true
            } else {
                false
            }
        })
    }
}
