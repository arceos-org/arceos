use alloc::sync::Arc;

use kernel_guard::{NoOp, NoPreemptIrqSave};
use kspin::{SpinNoIrq, SpinNoIrqGuard};
use linked_list::{def_node, List};

use crate::{current_run_queue, select_run_queue, AxTaskRef};
use crate::task::TaskState;

def_node!(WaitTaskNode, AxTaskRef);

macro_rules! declare_current_waiter {
    ($name: ident) => {
        let $name = Arc::new(WaitTaskNode::new($crate::current().clone()));
    };
}

type Node = Arc<WaitTaskNode>;

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
    list: SpinNoIrq<List<Node>>,
}

pub(crate) type WaitQueueGuard<'a> = SpinNoIrqGuard<'a, List<Node>>;

impl WaitQueue {
    /// Creates an empty wait queue.
    pub const fn new() -> Self {
        Self {
            list: SpinNoIrq::new(List::new()),
        }
    }

    fn waiter_blocked(&self, waiter: &Node) {
        let curr = waiter.inner();
        assert!(curr.is_running());
        assert!(!curr.is_idle());
        let mut wq_locked = self.list.lock();
        debug!("task block: {}", curr.id_name());
        // Mark the task as blocked, this has to be done before adding it to the wait queue
        // while holding the lock of the wait queue.
        curr.set_state(TaskState::Blocked);
        wq_locked.push_back(waiter.clone());
    }

    /// Cancel events by removing the task from the wait list.
    /// If `from_timer_list` is true, try to remove the task from the timer list.
    fn cancel_events(&self, waiter: &Node, _from_timer_list: bool) {
        // SAFETY:
        // Waiter is only  defined in the local function scope,
        // therefore, it is not possible in other List.
        unsafe {
            self.list.lock().remove(waiter);
        }

        // Try to cancel a timer event from timer lists.
        // Just mark task's current timer ticket ID as expired.
        #[cfg(feature = "irq")]
        if _from_timer_list {
            waiter.inner().timer_ticket_expired();
            // Note:
            //  this task is still not removed from timer list of target CPU,
            //  which may cause some redundant timer events because it still needs to
            //  go through the process of expiring an event from the timer list and invoking the callback.
            //  (it can be considered a lazy-removal strategy, it will be ignored when it is about to take effect.)
        }
    }

    /// Blocks the current task and put it into the wait list, until other task
    /// notifies it.
    pub fn wait(&self) {
        declare_current_waiter!(waiter);
        self.waiter_blocked(&waiter);
        current_run_queue::<NoPreemptIrqSave>().reschedule();
        self.cancel_events(&waiter, false);
    }

    /// Blocks the current task and put it into the wait list, until the given
    /// `condition` becomes true.
    ///
    /// Note that even other tasks notify this task, it will not wake up until
    /// the condition becomes true.
    pub fn wait_until<F>(&self, condition: F)
    where
        F: Fn() -> bool,
    {
        declare_current_waiter!(waiter);
        loop {
            if condition() {
                break;
            }
            self.waiter_blocked(&waiter);
            // Preemption may occur here.
            current_run_queue::<NoPreemptIrqSave>().reschedule();
        }

        self.cancel_events(&waiter, false);
    }

    /// Blocks the current task and put it into the wait list, until other tasks
    /// notify it, or the given duration has elapsed.
    #[cfg(feature = "irq")]
    pub fn wait_timeout(&self, dur: core::time::Duration) -> bool {
        declare_current_waiter!(waiter);
        let deadline = axhal::time::wall_time() + dur;
        debug!(
            "task wait_timeout: {} deadline={:?}",
            waiter.inner().id_name(),
            deadline
        );
        self.waiter_blocked(&waiter);
        crate::timers::set_alarm_wakeup(deadline, waiter.inner().clone());
        current_run_queue::<NoPreemptIrqSave>().reschedule();
        let timeout = axhal::time::wall_time() >= deadline;
        // Always try to remove the task from the timer list.
        self.cancel_events(&waiter, true);
        timeout
    }

    /// Blocks the current task and put it into the wait list, until the given
    /// `condition` becomes true, or the given duration has elapsed.
    ///
    /// Note that even other tasks notify this task, it will not wake up until
    /// the above conditions are met.
    #[cfg(feature = "irq")]
    pub fn wait_timeout_until<F>(&self, dur: core::time::Duration, condition: F) -> bool
    where
        F: Fn() -> bool,
    {
        declare_current_waiter!(waiter);
        let deadline = axhal::time::wall_time() + dur;
        debug!(
            "task wait_timeout: {}, deadline={:?}",
            waiter.inner().id_name(),
            deadline
        );
        crate::timers::set_alarm_wakeup(deadline, waiter.inner().clone());

        let mut timeout = true;
        loop {
            if axhal::time::wall_time() >= deadline {
                break;
            }
            if condition() {
                timeout = false;
                break;
            }

            self.waiter_blocked(&waiter);
            // Preemption may occur here.
            current_run_queue::<NoPreemptIrqSave>().reschedule();
        }

        // Always try to remove the task from the timer list.
        self.cancel_events(&waiter, true);
        timeout
    }

    /// Wakes up one task in the wait list, usually the first one.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_one(&self, resched: bool) -> bool {
        let mut wq = self.list.lock();
        if let Some(waiter) = wq.pop_front() {
            unblock_one_task(waiter.inner().clone(), resched);
            true
        } else {
            false
        }
    }

    /// Wakes all tasks in the wait list.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_all(&self, resched: bool) {
        while self.notify_one(resched) {
            // loop until the wait queue is empty
        }
    }

    /// Wake up the given task in the wait list.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_task(&mut self, resched: bool, task: &AxTaskRef) -> bool {
        let mut wq = self.list.lock();
        let mut cursor = wq.cursor_front_mut();
        loop {
            match cursor.current() {
                Some(node) => {
                    if Arc::ptr_eq(node.inner(), task) {
                        cursor.remove_current();
                        unblock_one_task(task.clone(), resched);
                        break true;
                    }
                }
                None => break false,
            }
            cursor.move_next();
        }
    } 
}

fn unblock_one_task(task: AxTaskRef, resched: bool) {
    // Select run queue by the CPU set of the task.
    // Use `NoOp` kernel guard here because the function is called with holding the
    // lock of wait queue, where the irq and preemption are disabled.
    select_run_queue::<NoOp>(&task).unblock_task(task, resched)
}
