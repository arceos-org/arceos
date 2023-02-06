use alloc::{collections::VecDeque, sync::Arc};
use core::ops::Deref;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::BaseScheduler;

const MAX_TIME_SLICE: usize = 10;

pub struct RRTask<T> {
    inner: T,
    time_slice: AtomicUsize,
}

impl<T> RRTask<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner,
            time_slice: AtomicUsize::new(MAX_TIME_SLICE),
        }
    }

    pub const fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T> const Deref for RRTask<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct RRScheduler<T> {
    ready_queue: VecDeque<Arc<RRTask<T>>>,
}

impl<T> RRScheduler<T> {
    pub const fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
}

impl<T> BaseScheduler for RRScheduler<T> {
    type SchedItem = Arc<RRTask<T>>;

    fn init(&mut self) {}

    fn add_task(&mut self, task: Self::SchedItem) {
        self.ready_queue.push_back(task);
    }

    fn remove_task(&mut self, task: &Self::SchedItem) -> Option<Self::SchedItem> {
        // TODO: more efficient
        self.ready_queue
            .iter()
            .position(|t| Arc::ptr_eq(t, task))
            .and_then(|idx| self.ready_queue.remove(idx))
    }

    fn pick_next_task(&mut self) -> Option<Self::SchedItem> {
        self.ready_queue.pop_front()
    }

    fn put_prev_task(&mut self, prev: Self::SchedItem) {
        self.ready_queue.push_back(prev)
    }

    fn task_tick(&mut self, current: &Self::SchedItem) -> bool {
        let old_slice = current.time_slice.fetch_sub(1, Ordering::Release);
        if old_slice == 1 {
            current.time_slice.store(MAX_TIME_SLICE, Ordering::Release);
            true
        } else {
            false
        }
    }
}
