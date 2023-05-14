use alloc::collections::VecDeque;
use alloc::sync::Arc;
use axhal::time::current_time;
use core::time::Duration;
use spinlock::SpinRaw;

use crate::{AxRunQueue, AxTaskRef, CurrentTask, RUN_QUEUE};

pub struct WaitQueue {
    queue: SpinRaw<VecDeque<AxTaskRef>>, // we already disabled IRQs when lock the `RUN_QUEUE`
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
    /// 取消等待
    fn cancel_events(&self, curr: CurrentTask) {
        // A task can be wake up only one events (timer or `notify()`), remove
        // the event from another queue.
        if curr.in_wait_queue() {
            // wake up by timer (timeout).
            // `RUN_QUEUE` is not locked here, so disable IRQs.
            let _guard = kernel_guard::IrqSave::new();
            self.queue.lock().retain(|t| !curr.ptr_eq(t));
            curr.set_in_wait_queue(false);
        }
        if curr.in_timer_list() {
            // timeout was set but not triggered (wake up by `WaitQueue::notify()`)
            crate::timers::cancel_alarm(curr.as_task_ref());
        }
    }

    pub fn wait(&self) {
        RUN_QUEUE.lock().block_current(|task| {
            task.set_in_wait_queue(true);
            self.queue.lock().push_back(task)
        });
        self.cancel_events(crate::current());
    }

    /// 正在wait的任务只能通过timer or `notify()`唤醒，所以gc任务回收无法唤醒正在睡眠的任务
    /// sleep系统调用会出现任务堵塞，所以wait_queue会被用到
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
        self.cancel_events(crate::current());
    }

    pub fn wait_timeout(&self, dur: Duration) -> bool {
        let curr = crate::current();
        let deadline = current_time() + dur;
        debug!(
            "task wait_timeout: {} deadline={:?}",
            curr.id_name(),
            deadline
        );
        crate::timers::set_alarm_wakeup(deadline, curr.clone());

        RUN_QUEUE.lock().block_current(|task| {
            task.set_in_wait_queue(true);
            self.queue.lock().push_back(task)
        });
        let timeout = curr.in_wait_queue(); // still in the wait queue, must have timed out
        self.cancel_events(curr);
        timeout
    }

    pub fn wait_timeout_until<F>(&self, dur: Duration, condition: F) -> bool
    where
        F: Fn() -> bool,
    {
        let curr = crate::current();
        let deadline = current_time() + dur;
        debug!(
            "task wait_timeout: {}, deadline={:?}",
            curr.id_name(),
            deadline
        );
        crate::timers::set_alarm_wakeup(deadline, curr.clone());

        let mut timeout = true;
        while current_time() < deadline {
            let mut rq = RUN_QUEUE.lock();
            if condition() {
                timeout = false;
                break;
            }
            rq.block_current(|task| {
                task.set_in_wait_queue(true);
                self.queue.lock().push_back(task);
            });
        }
        self.cancel_events(curr);
        timeout
    }

    pub fn notify_one(&self, resched: bool) -> bool {
        let mut rq = RUN_QUEUE.lock();
        if !self.queue.lock().is_empty() {
            self.notify_one_locked(resched, &mut rq)
        } else {
            false
        }
    }

    pub fn notify_all(&self, resched: bool) {
        loop {
            let mut rq = RUN_QUEUE.lock();
            if let Some(task) = self.queue.lock().pop_front() {
                task.set_in_wait_queue(false);
                rq.unblock_task(task, resched);
            } else {
                break;
            }
            drop(rq); // we must unlock `RUN_QUEUE` after unlocking `self.queue`.
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
