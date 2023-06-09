use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::cmp::Ord;
use core::ops::Deref;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::BaseScheduler;

/// RMSTASK
pub struct RMSTask<T> {
    inner: T,
    runtime: AtomicUsize,
    period: AtomicUsize,
}

impl<T> RMSTask<T> {
    /// new RMSTask 
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            runtime: 0.into(),
            period: 1.into(),
        }
    }

    /// Returns a reference to the inner task struct.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    fn set_period(&self, period: usize) {
        self.period.store(period, Ordering::Release);
    }

    fn set_runtime(&self, runtime: usize) {
        self.runtime.store(runtime, Ordering::Release);
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

/// RMS Scheduler
pub struct RMScheduler<T> {
    ready_queue: BTreeMap<TaskPriority, Arc<RMSTask<T>>>,
    next_task_id: AtomicUsize,
}

impl<T> RMScheduler<T> {
    /// new
    pub fn new() -> Self {
        Self {
            ready_queue: BTreeMap::new(),
            next_task_id: AtomicUsize::new(0),
        }
    }
    /// scheduler_name
    pub fn scheduler_name() -> &'static str {
        "RMS"
    }
}

impl<T> BaseScheduler for RMScheduler<T> {
    type SchedItem = Arc<RMSTask<T>>;

    fn init(&mut self) {}

    fn add_task(&mut self, task: Self::SchedItem) {
        let id = self.next_task_id.fetch_add(1, Ordering::Relaxed);
        let priority = TaskPriority {
            priority: 1 / task.period.load(Ordering::Acquire),
            id,
        };
        self.ready_queue.insert(priority, task);
    }

    fn remove_task(&mut self, task: &Self::SchedItem) -> Option<Self::SchedItem> {
        let priority_to_remove =
            self.ready_queue
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
            priority: 1 / prev.period.load(Ordering::Acquire),
            id,
        };
        self.ready_queue.insert(priority, prev);
    }

    fn task_tick(&mut self, _current: &Self::SchedItem) -> bool {
        // RMS 不使用时间片，我们不需要实现 task_tick
        false
    }

    fn set_priority(&mut self, task: &Self::SchedItem, prio: isize) -> bool {
        // 论如何用一个函数设置 rms 的优先级：
        // 正数表示 runtime, 负数表示 period
        if prio > 0 {
            task.set_runtime(prio as usize);
            true
        } else {
            task.set_period(-prio as usize);
            true
        }
    }

    fn is_empty(&self) -> bool {
        // 不允许线程窃取
        false
    }
}
