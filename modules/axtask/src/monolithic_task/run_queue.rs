use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_init::LazyInit;
use scheduler::BaseScheduler;
use spinlock::SpinNoIrq;

pub use crate::monolithic_task::task::{CurrentTask, TaskInner, TaskState, KERNEL_PROCESS_ID};

use crate::{AxTaskRef, Scheduler, WaitQueue};

// TODO: per-CPU
pub static RUN_QUEUE: LazyInit<SpinNoIrq<AxRunQueue>> = LazyInit::new();

// TODO: per-CPU
pub static EXITED_TASKS: SpinNoIrq<VecDeque<AxTaskRef>> = SpinNoIrq::new(VecDeque::new());

static WAIT_FOR_EXIT: WaitQueue = WaitQueue::new();

#[percpu::def_percpu]
pub static IDLE_TASK: LazyInit<AxTaskRef> = LazyInit::new();

pub struct AxRunQueue {
    scheduler: Scheduler,
}

impl AxRunQueue {
    pub fn new() -> SpinNoIrq<Self> {
        // 注意，此时是通过用户程序启动的，所以原有的arceos不要动
        let gc_task = TaskInner::new(
            gc_entry,
            "gc".into(),
            axconfig::TASK_STACK_SIZE,
            KERNEL_PROCESS_ID,
            0,
        );
        let mut scheduler = Scheduler::new();
        scheduler.add_task(gc_task);
        SpinNoIrq::new(Self { scheduler })
    }

    pub fn add_task(&mut self, task: AxTaskRef) {
        debug!("task spawn: {}", task.id_name());
        assert!(task.is_ready());
        self.scheduler.add_task(task);
    }

    /// 仅用于exec与exit时清除其他后台线程
    pub fn remove_task(&mut self, task: &AxTaskRef) {
        debug!("task remove: {}", task.id_name());
        // 当前任务不予清除
        assert!(task.is_running());
        assert!(!task.is_idle());
        self.scheduler.remove_task(task);
        task.set_state(TaskState::Exited);
        EXITED_TASKS.lock().push_back(task.clone());
    }

    #[cfg(feature = "irq")]
    pub fn scheduler_timer_tick(&mut self) {
        let curr = crate::current();
        if !curr.is_idle() && self.scheduler.task_tick(curr.as_task_ref()) {
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

    pub fn set_current_priority(&mut self, prio: isize) -> bool {
        self.scheduler
            .set_priority(crate::current().as_task_ref(), prio)
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

    pub fn exit_current(&mut self, exit_code: i32) {
        let curr = crate::current();
        debug!("task exit: {}, exit_code={}", curr.id_name(), exit_code);
        assert!(curr.is_running());
        assert!(!curr.is_idle());
        if curr.is_init() {
            EXITED_TASKS.lock().clear();
            axhal::misc::terminate();
        } else {
            curr.set_state(TaskState::Exited);
            curr.notify_exit(exit_code, self);
            EXITED_TASKS.lock().push_back(curr.clone());
            WAIT_FOR_EXIT.notify_one_locked(false, self);
            // 调度任务交给process进行
            // self.resched_inner(false);
        }
        // unreachable!("task exited!");
    }

    pub fn block_current<F>(&mut self, wait_queue_push: F)
    where
        F: FnOnce(AxTaskRef),
    {
        let curr = crate::current();
        debug!("task block: {}", curr.id_name());
        assert!(curr.is_running());
        assert!(!curr.is_idle());

        // we must not block current task with preemption disabled.
        #[cfg(feature = "preempt")]
        assert!(curr.can_preempt(1));

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

    #[cfg(feature = "irq")]
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
    pub fn resched_inner(&mut self, preempt: bool) {
        let prev = crate::current();
        if prev.is_running() {
            prev.set_state(TaskState::Ready);
            if !prev.is_idle() {
                self.scheduler.put_prev_task(prev.clone(), preempt);
            }
        }
        let next = self.scheduler.pick_next_task().unwrap_or_else(|| unsafe {
            // Safety: IRQs must be disabled at this time.
            IDLE_TASK.current_ref_raw().get_unchecked().clone()
        });
        self.switch_to(prev, next);
    }

    fn switch_to(&mut self, prev_task: CurrentTask, next_task: AxTaskRef) {
        trace!(
            "context switch: {} -> {}",
            prev_task.id_name(),
            next_task.id_name()
        );
        #[cfg(feature = "preempt")]
        next_task.set_preempt_pending(false);
        next_task.set_state(TaskState::Running);
        if prev_task.ptr_eq(&next_task) {
            return;
        }
        // 当任务进行切换时，更新两个任务的时间统计信息
        next_task.time_stat_when_switch_to();
        prev_task.time_stat_when_switch_from();
        unsafe {
            let prev_ctx_ptr = prev_task.ctx_mut_ptr();
            let next_ctx_ptr = next_task.ctx_mut_ptr();
            // The strong reference count of `prev_task` will be decremented by 1,
            // but won't be dropped until `gc_entry()` is called.
            assert!(Arc::strong_count(prev_task.as_task_ref()) > 1);
            assert!(Arc::strong_count(&next_task) >= 1);
            let page_table_token = next_task.page_table_token();
            if page_table_token != 0 {
                axhal::arch::write_page_table_root(page_table_token.into());
            }
            CurrentTask::set_current(prev_task, next_task);
            (*prev_ctx_ptr).switch_to(&*next_ctx_ptr);
        }
    }
}

fn gc_entry() {
    loop {
        // Drop all exited tasks and recycle resources.
        while !EXITED_TASKS.lock().is_empty() {
            // Do not do the slow drops in the critical section.
            let task = EXITED_TASKS.lock().pop_front();
            if let Some(task) = task {
                // If the task reference is not taken after `spawn()`, it will be
                // dropped here. Otherwise, it will be dropped after the reference
                // is dropped (usually by `join()`).
                drop(task);
            }
        }
        WAIT_FOR_EXIT.wait();
    }
}

pub(crate) fn init() {
    const IDLE_TASK_STACK_SIZE: usize = 4096;
    let idle_task = TaskInner::new(
        || crate::run_idle(),
        "idle".into(),
        IDLE_TASK_STACK_SIZE,
        KERNEL_PROCESS_ID,
        0,
    );
    IDLE_TASK.with_current(|i| i.init_by(idle_task.clone()));

    let main_task = TaskInner::new_init("main".into());
    main_task.set_state(TaskState::Running);

    RUN_QUEUE.init_by(AxRunQueue::new());
    unsafe { CurrentTask::init_current(main_task) }
}

pub(crate) fn init_secondary() {
    let idle_task = TaskInner::new_init("idle".into());
    idle_task.set_state(TaskState::Running);
    IDLE_TASK.with_current(|i| i.init_by(idle_task.clone()));
    unsafe { CurrentTask::init_current(idle_task) }
}
