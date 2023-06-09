use alloc::collections::VecDeque;
use alloc::sync::Arc;
use load_balance::BaseLoadBalance;
use spinlock::SpinRaw;

use crate::run_queue::{LOAD_BALANCE_ARR, RUN_QUEUE};
use crate::{get_current_cpu_id, AxRunQueue, AxTaskRef, CurrentTask};

use spinlock::SpinNoIrq;

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
    queue: SpinNoIrq<VecDeque<AxTaskRef>>, // we already disabled IRQs when lock the `RUN_QUEUE`
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
            // `RUN_QUEUE` is not locked here, so disable IRQs.
            let _guard = kernel_guard::IrqSave::new();
            self.queue.lock().retain(|t| !curr.ptr_eq(t));
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
        /*
         let tmp = if get_current_cpu_id() == axconfig::SMP {
            0
        } else {
            get_current_cpu_id()
        };
        let target_cpu = LOAD_BALANCE_ARR[tmp].find_target_cpu(crate::current().get_affinity());
        RUN_QUEUE[target_cpu].block_current(|task| {
            task.set_in_wait_queue(true);
            self.queue.lock().push_back(task)
        });
        */
        RUN_QUEUE[axhal::cpu::this_cpu_id()].with_current_rq(|rq| {
            rq.block_current(|task| {
                //task.set_in_wait_queue(true);
                self.queue.lock().push_back(task)
            });
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
            if condition() {
                break;
            }
            RUN_QUEUE[axhal::cpu::this_cpu_id()].with_current_rq(|rq| {
                rq.block_current(|task| {
                    //task.set_in_wait_queue(true);
                    self.queue.lock().push_back(task)
                });
            });
        }
        self.cancel_events(crate::current());
    }

    /// Blocks the current task and put it into the wait queue, until other tasks
    /// notify it, or the given duration has elapsed.
    #[cfg(feature = "irq")]
    pub fn wait_timeout(&self, dur: core::time::Duration) -> bool {
        let curr = crate::current();
        let deadline = axhal::time::current_time() + dur;
        debug!(
            "task wait_timeout: {} deadline={:?}",
            curr.id_name(),
            deadline
        );
        crate::timers::set_alarm_wakeup(deadline, curr.clone());
        RUN_QUEUE[axhal::cpu::this_cpu_id()].with_current_rq(|rq| {
            rq.block_current(|task| {
                //task.set_in_wait_queue(true);
                self.queue.lock().push_back(task)
            });
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
        let deadline = axhal::time::current_time() + dur;
        debug!(
            "task wait_timeout: {}, deadline={:?}",
            curr.id_name(),
            deadline
        );
        crate::timers::set_alarm_wakeup(deadline, curr.clone());

        let mut timeout = true;
        while axhal::time::current_time() < deadline {
            if condition() {
                timeout = false;
                break;
            }

            RUN_QUEUE[axhal::cpu::this_cpu_id()].with_current_rq(|rq| {
                rq.block_current(|task| {
                    //task.set_in_wait_queue(true);
                    self.queue.lock().push_back(task)
                });
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
        if !self.queue.lock().is_empty() {
            //let target_cpu = LOAD_BALANCE_ARR[get_current_cpu_id()].find_target_cpu(task.get_affinity());
            let tmp = self.notify_one_locked(resched);
            tmp
        } else {
            false
        }
    }

    /// Wakes all tasks in the wait queue.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_all(&self, resched: bool) {
        while let Some(task) = self.queue.lock().pop_front() {
            while !task.in_wait_queue() {}
            task.set_in_wait_queue(false);
            RUN_QUEUE[axhal::cpu::this_cpu_id()].with_task_correspond_rq(task.clone(), |rq| {
                rq.unblock_task(task, resched);
            });
        }
    }

    /// Wake up the given task in the wait queue.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_task(&self, resched: bool, task: &AxTaskRef) -> bool {
        let mut wq = self.queue.lock();
        if let Some(index) = wq.iter().position(|t| Arc::ptr_eq(t, task)) {
            task.set_in_wait_queue(false);
            // same as task
            let task_to_unblock = wq.remove(index).unwrap();
            RUN_QUEUE[axhal::cpu::this_cpu_id()].with_task_correspond_rq(
                task_to_unblock.clone(),
                |rq| {
                    rq.unblock_task(task_to_unblock, resched);
                },
            );
            true
        } else {
            false
        }
    }

    pub(crate) fn notify_one_locked(&self, resched: bool) -> bool {
        if let Some(task) = self.queue.lock().pop_front() {
            while !task.in_wait_queue() {}
            task.set_in_wait_queue(false);
            RUN_QUEUE[axhal::cpu::this_cpu_id()].with_task_correspond_rq(task.clone(), |rq| {
                rq.unblock_task(task, resched);
            });
            true
        } else {
            false
        }
    }

    pub(crate) fn notify_all_locked(&self, resched: bool, rq: &AxRunQueue) {
        while let Some(task) = self.queue.lock().pop_front() {
            while !task.in_wait_queue() {}
            task.set_in_wait_queue(false);
            rq.unblock_task(task, resched);
        }
    }
}
