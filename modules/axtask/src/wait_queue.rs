use alloc::collections::VecDeque;
use alloc::sync::Arc;
use spinlock::SpinRaw;

use crate::{AxRunQueue, AxTaskRef, RUN_QUEUE};

pub struct WaitQueue {
    queue: SpinRaw<VecDeque<AxTaskRef>>, // we already disabled IRQ when lock the `RUN_QUEUE
}

impl WaitQueue {
    pub const fn new() -> Self {
        Self {
            queue: SpinRaw::new(VecDeque::new()),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: SpinRaw::new(VecDeque::with_capacity(capacity)),
        }
    }

    fn cancel_events(&self, task: &AxTaskRef) {
        // A task can be wake up only one events (timer or `notify()`), remove
        // the event from another queue.
        if task.in_wait_queue() {
            // wake up by timer
            self.queue.lock().retain(|t| Arc::ptr_eq(t, task));
        }
    }

    pub fn wait(&self) {
        RUN_QUEUE.lock().block_current(|task| {
            task.set_in_wait_queue(true);
            self.queue.lock().push_back(task)
        });
        // may be woken up by other events rather than `notify_xxx()` (e.g. timer)
        self.cancel_events(crate::current());
    }

    pub fn wait_until<F>(&self, condition: F)
    where
        F: Fn() -> bool,
    {
        loop {
            let mut rq = RUN_QUEUE.lock();
            if condition() {
                break;
            }
            rq.block_current(|task| {
                task.set_in_wait_queue(true);
                self.queue.lock().push_back(task);
            });
        }
        // may be woken up by other events rather than `notify_xxx()` (e.g. timer)
        self.cancel_events(crate::current());
    }

    pub fn notify_one(&self, resched: bool) -> bool {
        if !self.queue.lock().is_empty() {
            self.notify_one_locked(resched, &mut RUN_QUEUE.lock())
        } else {
            false
        }
    }

    pub fn notify_all(&self, resched: bool) {
        if !self.queue.lock().is_empty() {
            let mut rq = RUN_QUEUE.lock();
            while let Some(task) = self.queue.lock().pop_front() {
                task.set_in_wait_queue(false);
                rq.unblock_task(task, resched);
            }
        }
    }

    pub fn notify_task(&mut self, resched: bool, task: &AxTaskRef) -> bool {
        let mut rq = RUN_QUEUE.lock();
        let mut wq = self.queue.lock();
        if let Some(index) = wq.iter().position(|t| Arc::ptr_eq(t, task)) {
            task.set_in_wait_queue(false);
            rq.unblock_task(wq.remove(index).unwrap(), resched);
            true
        } else {
            false
        }
    }

    pub(crate) fn notify_one_locked(&self, resched: bool, rq: &mut AxRunQueue) -> bool {
        if let Some(task) = self.queue.lock().pop_front() {
            task.set_in_wait_queue(false);
            rq.unblock_task(task, resched);
            true
        } else {
            false
        }
    }
}
