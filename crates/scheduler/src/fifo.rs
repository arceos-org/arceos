use alloc::sync::Arc;
use core::ops::Deref;

use linked_list::{Adapter, Links, List};

use crate::BaseScheduler;

pub struct FifoTask<T> {
    inner: T,
    links: Links<Self>,
}

unsafe impl<T> Adapter for FifoTask<T> {
    type EntryType = Self;

    #[inline]
    fn to_links(t: &Self) -> &Links<Self> {
        &t.links
    }
}

impl<T> FifoTask<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner,
            links: Links::new(),
        }
    }

    pub const fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T> const Deref for FifoTask<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct FifoScheduler<T> {
    ready_queue: List<Arc<FifoTask<T>>>,
}

impl<T> FifoScheduler<T> {
    pub const fn new() -> Self {
        Self {
            ready_queue: List::new(),
        }
    }
}

impl<T> BaseScheduler for FifoScheduler<T> {
    type SchedItem = Arc<FifoTask<T>>;

    fn init(&mut self) {}

    fn add_task(&mut self, task: Self::SchedItem) {
        self.ready_queue.push_back(task);
    }

    fn remove_task(&mut self, task: &Self::SchedItem) -> Option<Self::SchedItem> {
        unsafe { self.ready_queue.remove(task) }
    }

    fn pick_next_task(&mut self) -> Option<Self::SchedItem> {
        self.ready_queue.pop_front()
    }

    fn put_prev_task(&mut self, prev: Self::SchedItem) {
        self.ready_queue.push_back(prev);
    }

    fn task_tick(&mut self, _current: &Self::SchedItem) -> bool {
        false // no reschedule
    }
}
