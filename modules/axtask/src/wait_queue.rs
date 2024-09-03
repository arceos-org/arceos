use alloc::collections::VecDeque;
use alloc::sync::Arc;
use kspin::SpinNoIrq;

use crate::{current_run_queue, select_run_queue, AxTaskRef, CurrentTask};

/// A queue to store sleeping tasks.
///
/// # Examples
///
/// ```
/// use axtask::WaitQueue;
/// use core::sync::atomic::{AtomicU32, Ordering};
///
/// static VALUE: AtomicU32 = AtomicU32::new(0);
/// static WQ: WaitQueue = WaitQueue::new();
///
/// axtask::init_scheduler();
/// // spawn a new task that updates `VALUE` and notifies the main task
/// axtask::spawn(|| {
///     assert_eq!(VALUE.load(Ordering::Relaxed), 0);
///     VALUE.fetch_add(1, Ordering::Relaxed);
///     WQ.notify_one(true); // wake up the main task
/// });
///
/// WQ.wait(); // block until `notify()` is called
/// assert_eq!(VALUE.load(Ordering::Relaxed), 1);
/// ```
pub struct WaitQueue {
    queue: SpinNoIrq<VecDeque<AxTaskRef>>,
}

impl WaitQueue {
    /// Creates an empty wait queue.
    pub const fn new() -> Self {
        Self {
            queue: SpinNoIrq::new(VecDeque::new()),
        }
    }

    /// Creates an empty wait queue with space for at least `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: SpinNoIrq::new(VecDeque::with_capacity(capacity)),
        }
    }

    fn cancel_events(&self, curr: CurrentTask) {
        // A task can be wake up only one events (timer or `notify()`), remove
        // the event from another queue.
        if curr.in_wait_queue() {
            // wake up by timer (timeout).
            let mut wait_queue_locked = self.queue.lock();
            wait_queue_locked.retain(|t| !curr.ptr_eq(t));
            curr.set_in_wait_queue(false);
        }
        #[cfg(feature = "irq")]
        if curr.in_timer_list() {
            // timeout was set but not triggered (wake up by `WaitQueue::notify()`)
            crate::timers::cancel_alarm(curr.as_task_ref());
        }
    }

    /// Blocks the current task and put it into the wait queue, until other task
    /// notifies it.
    pub fn wait(&self) {
        current_run_queue()
            .scheduler()
            .lock()
            .block_current(|task| {
                task.set_in_wait_queue(true);
                self.queue.lock().push_back(task)
            });
        self.cancel_events(crate::current());
    }

    /// Blocks the current task and put it into the wait queue, until the given
    /// `condition` becomes true.
    ///
    /// Note that even other tasks notify this task, it will not wake up until
    /// the condition becomes true.
    pub fn wait_until<F>(&self, condition: F)
    where
        F: Fn() -> bool,
    {
        loop {
            let mut rq_locked = current_run_queue().scheduler().lock();
            if condition() {
                break;
            }
            rq_locked.block_current(|task| {
                task.set_in_wait_queue(true);
                self.queue.lock().push_back(task)
            });
        }
        self.cancel_events(crate::current());
    }

    /// Blocks the current task and put it into the wait queue, until other tasks
    /// notify it, or the given duration has elapsed.
    #[cfg(feature = "irq")]
    pub fn wait_timeout(&self, dur: core::time::Duration) -> bool {
        let mut rq_locked = current_run_queue().scheduler().lock();
        let curr = crate::current();
        let deadline = axhal::time::wall_time() + dur;
        debug!(
            "task wait_timeout: {} deadline={:?}",
            curr.id_name(),
            deadline
        );
        crate::timers::set_alarm_wakeup(deadline, curr.clone());

        rq_locked.block_current(|task| {
            task.set_in_wait_queue(true);
            self.queue.lock().push_back(task)
        });
        let timeout = curr.in_wait_queue(); // still in the wait queue, must have timed out
        self.cancel_events(curr);
        timeout
    }

    /// Blocks the current task and put it into the wait queue, until the given
    /// `condition` becomes true, or the given duration has elapsed.
    ///
    /// Note that even other tasks notify this task, it will not wake up until
    /// the above conditions are met.
    #[cfg(feature = "irq")]
    pub fn wait_timeout_until<F>(&self, dur: core::time::Duration, condition: F) -> bool
    where
        F: Fn() -> bool,
    {
        let curr = crate::current();
        let deadline = axhal::time::wall_time() + dur;
        debug!(
            "task wait_timeout: {}, deadline={:?}",
            curr.id_name(),
            deadline
        );
        crate::timers::set_alarm_wakeup(deadline, curr.clone());

        let mut timeout = true;
        while axhal::time::wall_time() < deadline {
            let mut rq_locked = current_run_queue().scheduler().lock();
            if condition() {
                timeout = false;
                break;
            }
            rq_locked.block_current(|task| {
                task.set_in_wait_queue(true);
                self.queue.lock().push_back(task)
            });
        }
        self.cancel_events(curr);
        timeout
    }

    /// Wakes up one task in the wait queue, usually the first one.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_one(&self, resched: bool) -> bool {
        let Some(task) = self.queue.lock().pop_front() else {
            return false;
        };
        unblock_one_task(task, resched);
        true
    }

    /// Wakes all tasks in the wait queue.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_all(&self, resched: bool) {
        loop {
            let Some(task) = self.queue.lock().pop_front() else {
                break;
            };
            unblock_one_task(task, resched);
        }
    }

    /// Wake up the given task in the wait queue.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_task(&mut self, resched: bool, task: &AxTaskRef) -> bool {
        let task_to_be_notify = {
            let mut wq = self.queue.lock();
            if let Some(index) = wq.iter().position(|t| Arc::ptr_eq(t, task)) {
                wq.remove(index)
            } else {
                None
            }
        };
        if let Some(task) = task_to_be_notify {
            unblock_one_task(task, resched);
            true
        } else {
            false
        }
    }
}

pub(crate) fn unblock_one_task(task: AxTaskRef, resched: bool) {
    // Select run queue by the CPU set of the task.
    let mut rq_locked = select_run_queue(
        #[cfg(feature = "smp")]
        task.clone(),
    )
    .scheduler()
    .lock();

    task.set_in_wait_queue(false);
    rq_locked.unblock_task(task, resched)
}
