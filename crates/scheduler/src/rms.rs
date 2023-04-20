use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::ops::Deref;
use core::cmp::Ord;
use alloc::sync::Arc;

use crate::BaseScheduler;

pub struct RMSTask<T> {
    inner: T,
    runtime: usize,
    period: usize,
}

impl<T> RMSTask<T> {
    pub fn new(inner: T, runtime: usize, period: usize) -> Self {
        Self {
            inner,
            runtime,
            period,
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T> Deref for RMSTask<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct TaskPriority {
    priority: usize,
    id: usize,
}

pub struct RMScheduler<T> {
    ready_queue: BTreeMap<TaskPriority, Arc<RMSTask<T>>>,
    next_task_id: AtomicUsize,
}

impl<T> RMScheduler<T> {
    pub fn new() -> Self {
        Self {
            ready_queue: BTreeMap::new(),
            next_task_id: AtomicUsize::new(0),
        }
    }
}

impl<T> BaseScheduler for RMScheduler<T> {
    type SchedItem = Arc<RMSTask<T>>;

    fn init(&mut self) {}

    fn add_task(&mut self, task: Self::SchedItem) {
        let id = self.next_task_id.fetch_add(1, Ordering::Relaxed);
        let priority = TaskPriority {
            priority: 1 / task.period,
            id,
        };
        self.ready_queue.insert(priority, task);
    }

    fn remove_task(&mut self, task: &Self::SchedItem) -> Option<Self::SchedItem> {
        let priority_to_remove = self.ready_queue
            .iter()
            .find_map(|(p, t)| if Arc::ptr_eq(t, task) { Some(*p) } else { None });
    
        if let Some(priority) = priority_to_remove {
            self.ready_queue.remove(&priority)
        } else {
            None
        }
    }
    
    fn pick_next_task(&mut self) -> Option<Self::SchedItem> {
        let highest_priority = self.ready_queue.keys().rev().next().cloned();
    
        if let Some(priority) = highest_priority {
            self.ready_queue.remove(&priority)
        } else {
            None
        }
    }

    fn put_prev_task(&mut self, prev: Self::SchedItem, _preempt: bool) {
        // 将任务放回队列
        let id = self.next_task_id.fetch_add(1, Ordering::Relaxed);
        let priority = TaskPriority {
            priority: 1 / prev.period,
            id,
        };
        self.ready_queue.insert(priority, prev);
    }

    fn task_tick(&mut self, _current: &Self::SchedItem) -> bool {
        // RMS 不使用时间片，我们不需要实现 task_tick
        false
    }
}