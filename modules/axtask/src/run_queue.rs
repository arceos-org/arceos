use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_init::LazyInit;
use scheduler::BaseScheduler;
use spinlock::SpinNoIrq;

use crate::task::{CurrentTask, TaskState};
use crate::{run_idle, AxTaskRef, Scheduler, TaskInner, WaitQueue};

const KERNEL_PROCESS_ID: u64 = 1;

// TODO: per-CPU
pub static RUN_QUEUE: LazyInit<SpinNoIrq<AxRunQueue>> = LazyInit::new();

// TODO: per-CPU
static EXITED_TASKS: SpinNoIrq<VecDeque<AxTaskRef>> = SpinNoIrq::new(VecDeque::new());

static WAIT_FOR_EXIT: WaitQueue = WaitQueue::new();
const IDLE_TASK_STACK_SIZE: usize = 4096;
#[percpu::def_percpu]
pub static IDLE_TASK: LazyInit<AxTaskRef> = LazyInit::new();

pub struct AxRunQueue {
    scheduler: Scheduler,
}

impl AxRunQueue {
    pub fn new() -> SpinNoIrq<Self> {
        // 内核线程的page_table_token默认为0
        let gc_task = TaskInner::new(
            gc_entry,
            "gc",
            axconfig::TASK_STACK_SIZE,
            KERNEL_PROCESS_ID,
            0,
        );
        info!("gc task id: {}", gc_task.id().as_u64());
        unsafe { CurrentTask::init_current(gc_task) }
        let scheduler = Scheduler::new();
        // scheduler.add_task(gc_task);
        SpinNoIrq::new(Self { scheduler })
    }

    pub fn add_task(&mut self, task: AxTaskRef) {
        debug!("task spawn: {}", task.id_name());
        assert!(task.is_ready());
        self.scheduler.add_task(task);
    }
    /// 仅用于exec时清除其他后台线程
    pub fn remove_task(&mut self, task: &AxTaskRef) {
        debug!("task remove: {}", task.id_name());
        // 当前任务不予清除
        assert!(task.is_running());
        assert!(!task.is_idle());
        self.scheduler.remove_task(task);
        task.set_state(TaskState::Exited);
        EXITED_TASKS.lock().push_back(task.clone());
    }

    pub fn scheduler_timer_tick(&mut self) {
        let curr = crate::current();
        if !curr.is_idle() && self.scheduler.task_tick(curr.as_task_ref()) {
            #[cfg(feature = "preempt")]
            curr.set_preempt_pending(true);
        }
    }

    pub fn yield_current(&mut self) {
        let curr = crate::current();
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
    /// 线程的退出，需要判断当前线程是否为调度线程。
    /// 若是调度线程，则需要退出进程内所有程序
    pub fn exit_current(&mut self, exit_code: i32) {
        let curr = crate::current();
        debug!("task exit: {}, exit_code={}", curr.id_name(), exit_code);
        assert!(curr.is_running());
        assert!(!curr.is_idle());
        if curr.is_init() {
            EXITED_TASKS.lock().clear();
            axhal::misc::terminate();
        } else {
            curr.set_exit_code(exit_code);
            curr.set_state(TaskState::Exited);
            EXITED_TASKS.lock().push_back(curr.clone());
            WAIT_FOR_EXIT.notify_one_locked(false, self);
        }
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
                // wait for other threads to release the reference.
                while Arc::strong_count(&task) > 1 {
                    core::hint::spin_loop();
                }
                drop(task);
            }
        }
        WAIT_FOR_EXIT.wait();
    }
}

pub(crate) fn init() {
    let idle_task = TaskInner::new(
        || run_idle(),
        "idle",
        IDLE_TASK_STACK_SIZE,
        KERNEL_PROCESS_ID,
        0,
    );
    idle_task.set_leader(true);
    IDLE_TASK.with_current(|i| i.init_by(idle_task.clone()));
    RUN_QUEUE.init_by(AxRunQueue::new());
}

/// 副核启动
pub(crate) fn init_secondary() {
    let idle_task = TaskInner::new_init("idle");
    idle_task.set_state(TaskState::Running);
    IDLE_TASK.with_current(|i| i.init_by(idle_task.clone()));
    unsafe { CurrentTask::init_current(idle_task) }
}
