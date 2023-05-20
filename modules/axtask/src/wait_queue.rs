use alloc::collections::VecDeque;
use alloc::sync::Arc;
use load_balance::BaseLoadBalance;
use spinlock::SpinRaw;

use crate::run_queue::LOAD_BALANCE_ARR;
use crate::{AxRunQueue, AxTaskRef, CurrentTask, RUN_QUEUE};

use crate::get_current_cpu_id;

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
    queue: SpinRaw<VecDeque<(AxTaskRef, usize)>>, // we already disabled IRQs when lock the `RUN_QUEUE`
}

impl WaitQueue {
    /// Creates an empty wait queue.
    pub const fn new() -> Self {
        Self {
            queue: SpinRaw::new(VecDeque::new()),
        }
    }

    /// Creates an empty wait queue with space for at least `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: SpinRaw::new(VecDeque::with_capacity(capacity)),
        }
    }

    fn cancel_events(&self, curr: CurrentTask) {
        // A task can be wake up only one events (timer or `notify()`), remove
        // the event from another queue.
        if curr.in_wait_queue() {
            // wake up by timer (timeout).
            // `RUN_QUEUE` is not locked here, so disable IRQs.
            let _guard = kernel_guard::IrqSave::new();
            self.queue.lock().retain(|(t, _)| !curr.ptr_eq(t));
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
        debug!("lock begin 7");
        RUN_QUEUE[get_current_cpu_id()].block_current(|task| {
            task.set_in_wait_queue(true);
            self.queue.lock().push_back((task, get_current_cpu_id()))
        });
        debug!("lock end 7");
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
            RUN_QUEUE[get_current_cpu_id()].block_current(|task| {
                task.set_in_wait_queue(true);
                self.queue.lock().push_back((task, get_current_cpu_id()));
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

        debug!("lock begin 5");
        RUN_QUEUE[get_current_cpu_id()].block_current(|task| {
            task.set_in_wait_queue(true);
            self.queue.lock().push_back((task, get_current_cpu_id()))
        });
        debug!("lock end 5");
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
        assert!(false);
        let curr = crate::current();
        let deadline = axhal::time::current_time() + dur;
        debug!(
            "until: task wait_timeout: {}, deadline={:?}",
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
            RUN_QUEUE[get_current_cpu_id()].block_current(|task| {
                task.set_in_wait_queue(true);
                self.queue.lock().push_back((task, get_current_cpu_id()));
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
        //info!("tat {}", get_current_cpu_id());
        debug!("lock begin 3");
        if !self.queue.lock().is_empty() {
            //info!("tat1 {}", get_current_cpu_id());
            let target_cpu = LOAD_BALANCE_ARR[get_current_cpu_id()].find_target_cpu();
            let tmp = self.notify_one_locked(resched, &RUN_QUEUE[target_cpu], target_cpu);
            //debug!("lock end 3");
            tmp
        } else {
            //info!("tat2 {}", get_current_cpu_id());
            //debug!("lock end 3");
            false
        }
    }

    /// Wakes all tasks in the wait queue.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_all(&self, resched: bool) {
        loop {
            //info!("333");
            if let Some((task, cpu_id)) = self.queue.lock().pop_front() {
                task.set_in_wait_queue(false);
                RUN_QUEUE[cpu_id].unblock_task(task, resched);
                //info!("exit 2");
                //drop(rq); // we must unlock `RUN_QUEUE` after unlocking `self.queue`.
            } else {
                break;
            }
        }
    }

    /// Wake up the given task in the wait queue.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_task(&self, resched: bool, task: &AxTaskRef) -> bool {
        //info!("222");
        let mut wq = self.queue.lock();
        if let Some(index) = wq.iter().position(|(t, cpu_id)| Arc::ptr_eq(t, task)) {
            task.set_in_wait_queue(false);
            let target_cpu = LOAD_BALANCE_ARR[get_current_cpu_id()].find_target_cpu();
            let task = wq.remove(index).unwrap().0;
            task.set_queue_id(target_cpu);
            RUN_QUEUE[target_cpu].unblock_task(task, resched);
            //info!("exit 4");
            true
        } else {
            false
        }
    }

    pub(crate) fn notify_one_locked(&self, resched: bool, rq: &AxRunQueue, queueid: usize) -> bool {
        //assert!(false);
        let tmp = self.queue.lock();
        //info!("111 {} {}", get_current_cpu_id(), tmp[0].1);
        for i in 0..tmp.len() {
            tmp[i].0.set_in_wait_queue(false);
            tmp[i].0.set_queue_id(queueid);
            rq.unblock_task(tmp[i].0.clone(), resched);
            drop(tmp);
            return true;
        }
        false
    }
}
