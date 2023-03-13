use alloc::{sync::Arc, vec::Vec};
use lazy_init::LazyInit;
use scheduler::BaseScheduler;
use spinlock::{SpinNoIrq, SpinNoPreempt};

use crate::task::TaskState;
use crate::{AxTaskRef, Scheduler, TaskInner, WaitQueue};

const BUILTIN_TASK_STACK_SIZE: usize = 4096;

// TODO: per-CPU
pub(crate) static RUN_QUEUE: LazyInit<SpinNoIrq<AxRunQueue>> = LazyInit::new();

// TODO: per-CPU
static EXITED_TASKS: SpinNoPreempt<Vec<AxTaskRef>> = SpinNoPreempt::new(Vec::new());

static WAIT_FOR_EXIT: WaitQueue = WaitQueue::new();

pub(crate) struct AxRunQueue {
    idle_task: AxTaskRef,
    init_task: AxTaskRef,
    scheduler: Scheduler,
}

impl AxRunQueue {
    pub fn new() -> SpinNoIrq<Self> {
        let idle_task = TaskInner::new_idle(BUILTIN_TASK_STACK_SIZE);
        let init_task = TaskInner::new_init();
        let gc_task = TaskInner::new(gc_entry, "gc", BUILTIN_TASK_STACK_SIZE);
        let mut scheduler = Scheduler::new();
        scheduler.add_task(gc_task);
        SpinNoIrq::new(Self {
            idle_task,
            init_task,
            scheduler,
        })
    }

    pub fn init_task(&self) -> &AxTaskRef {
        &self.init_task
    }

    pub fn add_task(&mut self, task: AxTaskRef) {
        debug!("task spawn: {}", task.id_name());
        assert!(task.is_ready());
        self.scheduler.add_task(task);
    }

    pub fn scheduler_timer_tick(&mut self) {
        let curr = crate::current();
        if !curr.is_idle() && self.scheduler.task_tick(curr) {
            #[cfg(feature = "preempt")]
            curr.set_preempt_pending(true);
        }
    }

    pub fn yield_current(&mut self) {
        let curr = crate::current();
        debug!("task yield: {}", curr.id_name());
        assert!(curr.is_running());
        self.resched_inner(false);
    }

    #[cfg(feature = "preempt")]
    pub fn resched(&mut self) {
        let curr = crate::current();
        assert!(curr.is_running());

        // When we get the mutable reference of the run queue, we must
        // have held the `SpinNoIrq` lock with both IRQs and preemption
        // disabled. So we need to set `current_disable_count` to 1 in
        // `can_preempt()` to obtain the preemption permission before
        //  locking the run queue.
        let can_preempt = curr.can_preempt(1);

        debug!(
            "current task is to be preempted: {}, allow={}",
            curr.id_name(),
            can_preempt
        );
        if can_preempt {
            self.resched_inner(true);
        } else {
            curr.set_preempt_pending(true);
        }
    }

    pub fn exit_current(&mut self, exit_code: i32) -> ! {
        let curr = crate::current();
        debug!("task exit: {}, exit_code={}", curr.id_name(), exit_code);
        assert!(curr.is_running());
        assert!(!curr.is_idle());
        if Arc::ptr_eq(curr, &self.init_task) {
            EXITED_TASKS.lock().clear();
            axhal::misc::terminate();
        } else {
            curr.set_state(TaskState::Exited);
            EXITED_TASKS.lock().push(curr.clone());
            WAIT_FOR_EXIT.notify_one_locked(false, self);
            self.resched_inner(false);
        }
        unreachable!("task exited!");
    }

    pub fn block_current<F>(&mut self, wait_queue_push: F)
    where
        F: FnOnce(AxTaskRef),
    {
        let curr = crate::current();

        // we must not block current task with preemption disabled.
        #[cfg(feature = "preempt")]
        assert!(curr.can_preempt(1));

        debug!("task block: {}", curr.id_name());
        assert!(curr.is_running());
        assert!(!curr.is_idle());
        curr.set_state(TaskState::Blocked);
        wait_queue_push(curr.clone());
        self.resched_inner(false);
    }

    pub fn unblock_task(&mut self, task: AxTaskRef, resched: bool) {
        debug!("task unblock: {}", task.id_name());
        if task.is_blocked() {
            task.set_state(TaskState::Ready);
            self.scheduler.add_task(task); // TODO: priority
            if resched {
                #[cfg(feature = "preempt")]
                crate::current().set_preempt_pending(true);
            }
        }
    }

    pub fn sleep_until(&mut self, deadline: axhal::time::TimeValue) {
        let curr = crate::current();
        debug!("task sleep: {}, deadline={:?}", curr.id_name(), deadline);
        assert!(curr.is_running());
        assert!(!curr.is_idle());

        let now = axhal::time::current_time();
        if now < deadline {
            crate::timers::set_alarm_wakeup(deadline, curr.clone());
            curr.set_state(TaskState::Blocked);
            self.resched_inner(false);
        }
    }
}

impl AxRunQueue {
    /// Common reschedule subroutine. If `preempt`, keep current task's time
    /// slice, otherwise reset it.
    fn resched_inner(&mut self, preempt: bool) {
        let prev = crate::current();
        if prev.is_running() {
            prev.set_state(TaskState::Ready);
            if !prev.is_idle() {
                self.scheduler.put_prev_task(prev.clone(), preempt);
            }
        }
        let next = self
            .scheduler
            .pick_next_task()
            .unwrap_or_else(|| self.idle_task.clone());
        self.switch_to(prev, next);
    }

    fn switch_to(&mut self, prev_task: &AxTaskRef, next_task: AxTaskRef) {
        trace!(
            "context switch: {} -> {}",
            prev_task.id_name(),
            next_task.id_name()
        );
        #[cfg(feature = "preempt")]
        next_task.set_preempt_pending(false);
        next_task.set_state(TaskState::Running);
        if Arc::ptr_eq(prev_task, &next_task) {
            return;
        }

        unsafe {
            let prev_ctx_ptr = prev_task.ctx_mut_ptr();
            let next_ctx_ptr = next_task.ctx_mut_ptr();

            // The strong reference count of `prev_task` will be decremented by 1,
            // but won't be dropped until `gc_function()` is called.
            assert!(Arc::strong_count(prev_task) > 1);
            assert!(Arc::strong_count(&next_task) >= 1);

            crate::set_current(next_task);
            (*prev_ctx_ptr).switch_to(&*next_ctx_ptr);
        }
    }
}

fn gc_entry() {
    loop {
        // Drop all exited tasks and recycle resources.
        // Do not do the slow drops in the critical section.
        while !EXITED_TASKS.lock().is_empty() {
            let task = EXITED_TASKS.lock().pop();
            drop(task);
        }
        WAIT_FOR_EXIT.wait();
    }
}
