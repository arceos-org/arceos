use alloc::collections::VecDeque;
use alloc::sync::Arc;

use kernel_guard::NoPreemptIrqSave;
use kspin::SpinNoIrq;

use crate::{current_run_queue, select_run_queue, task::TaskState, AxTaskRef, CurrentTask};

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

    /// Cancel events by removing the task from the wait queue.
    /// If `from_timer_list` is true, the task should be removed from the timer list.
    fn cancel_events(&self, curr: CurrentTask, from_timer_list: bool) {
        // A task can be wake up only one events (timer or `notify()`), remove
        // the event from another queue.
        if curr.in_wait_queue() {
            // wake up by timer (timeout).
            let mut wq_locked = self.queue.lock();
            wq_locked.retain(|t| !curr.ptr_eq(t));
            curr.set_in_wait_queue(false);
        }
        // Try to cancel a timer event from timer lists.
        // Just mark task's current timer ticket ID as expired.
        #[cfg(feature = "irq")]
        if from_timer_list {
            curr.timer_ticket_expire_one();
            // TODO:
            // this task is still not removed from timer list of target CPU,
            // which may cause some redundant timer events.
        }
    }

    fn push_to_wait_queue(&self) {
        let mut wq = self.queue.lock();
        let curr = crate::current();
        assert!(curr.is_running());
        assert!(!curr.is_idle());
        // we must not block current task with preemption disabled.
        // Current expected preempt count is 2.
        // 1 for `NoPreemptIrqSave`, 1 for wait queue's `SpinNoIrq`.
        #[cfg(feature = "preempt")]
        assert!(curr.can_preempt(2));

        curr.set_state(TaskState::Blocked);
        curr.set_in_wait_queue(true);

        debug!("{} push to wait queue", curr.id_name());

        wq.push_back(curr.clone());
    }

    /// Blocks the current task and put it into the wait queue, until other task
    /// notifies it.
    pub fn wait(&self) {
        let mut rq = current_run_queue::<NoPreemptIrqSave>();
        self.push_to_wait_queue();
        rq.blocked_resched();
        self.cancel_events(crate::current(), false);
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
        let mut rq = current_run_queue::<NoPreemptIrqSave>();
        loop {
            let mut wq = self.queue.lock();
            if condition() {
                break;
            }
            let curr = crate::current();
            assert!(curr.is_running());
            assert!(!curr.is_idle());

            debug!("{} push to wait queue on wait_until", curr.id_name());

            // we must not block current task with preemption disabled.
            // Current expected preempt count is 2.
            // 1 for `NoPreemptIrqSave`, 1 for wait queue's `SpinNoIrq`.
            #[cfg(feature = "preempt")]
            assert!(curr.can_preempt(2));
            wq.push_back(curr.clone());

            curr.set_state(TaskState::Blocked);
            curr.set_in_wait_queue(true);
            drop(wq);

            rq.blocked_resched();
        }
        self.cancel_events(crate::current(), false);
    }

    /// Blocks the current task and put it into the wait queue, until other tasks
    /// notify it, or the given duration has elapsed.
    #[cfg(feature = "irq")]
    pub fn wait_timeout(&self, dur: core::time::Duration) -> bool {
        let mut rq = current_run_queue::<NoPreemptIrqSave>();
        let curr = crate::current();
        let deadline = axhal::time::wall_time() + dur;
        debug!(
            "task wait_timeout: {} deadline={:?}",
            curr.id_name(),
            deadline
        );
        crate::timers::set_alarm_wakeup(deadline, curr.clone());

        self.push_to_wait_queue();
        rq.blocked_resched();

        let timeout = curr.in_wait_queue(); // still in the wait queue, must have timed out

        // If `timeout` is true, the task is still in the wait queue,
        // which means timer event is triggered and the task has been removed from timer list.
        self.cancel_events(curr, !timeout);
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
        let mut rq = current_run_queue::<NoPreemptIrqSave>();
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
            let mut wq = self.queue.lock();
            if condition() {
                timeout = false;
                break;
            }
            assert!(curr.is_running());
            assert!(!curr.is_idle());

            // we must not block current task with preemption disabled.
            // Current expected preempt count is 2.
            // 1 for `NoPreemptIrqSave`, 1 for wait queue's `SpinNoIrq`.
            #[cfg(feature = "preempt")]
            assert!(curr.can_preempt(2));
            wq.push_back(curr.clone());

            curr.set_state(TaskState::Blocked);
            curr.set_in_wait_queue(true);
            drop(wq);

            rq.blocked_resched()
        }
        self.cancel_events(curr, !timeout);
        timeout
    }

    /// Wakes up one task in the wait queue, usually the first one.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_one(&self, resched: bool) -> bool {
        let mut wq = self.queue.lock();
        if let Some(task) = wq.pop_front() {
            task.set_in_wait_queue(false);
            unblock_one_task(task, resched);
            drop(wq);
            true
        } else {
            false
        }
    }

    /// Wakes all tasks in the wait queue.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_all(&self, resched: bool) {
        loop {
            let mut wq = self.queue.lock();
            if let Some(task) = wq.pop_front() {
                task.set_in_wait_queue(false);
                unblock_one_task(task, resched);
            } else {
                break;
            }
            drop(wq);
        }
    }

    /// Wake up the given task in the wait queue.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_task(&mut self, resched: bool, task: &AxTaskRef) -> bool {
        let mut wq = self.queue.lock();
        let task_to_be_notify = {
            if let Some(index) = wq.iter().position(|t| Arc::ptr_eq(t, task)) {
                wq.remove(index)
            } else {
                None
            }
        };
        if let Some(task) = task_to_be_notify {
            // Mark task as not in wait queue.
            task.set_in_wait_queue(false);
            unblock_one_task(task, resched);
            drop(wq);
            true
        } else {
            false
        }
    }
}

pub(crate) fn unblock_one_task(task: AxTaskRef, resched: bool) {
    // Select run queue by the CPU set of the task.
    select_run_queue::<NoPreemptIrqSave>(task.clone()).unblock_task(task, resched)
}
