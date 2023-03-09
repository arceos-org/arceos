use alloc::{sync::Arc, vec::Vec};
use lazy_init::LazyInit;
use scheduler::BaseScheduler;
use spinlock::SpinNoIrq;

use crate::task::TaskState;
use crate::{AxTaskRef, Scheduler, TaskInner, WaitQueue};

const BUILTIN_TASK_STACK_SIZE: usize = 4096;

// TODO: per-CPU
pub(crate) static RUN_QUEUE: LazyInit<SpinNoIrq<AxRunQueue>> = LazyInit::new();

// TODO: per-CPU
static EXITED_TASKS: SpinNoIrq<Vec<AxTaskRef>> = SpinNoIrq::new(Vec::new());

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
        scheduler.add_task(init_task.clone());
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
        self.scheduler.add_task(task);
    }

    pub fn scheduler_timer_tick(&mut self) {
        let curr = crate::current();
        if self.scheduler.task_tick(curr) {
            curr.set_need_resched();
        }
    }

    pub fn yield_current(&mut self) {
        let task = crate::current();
        debug!("task yield: {}", task.id_name());
        assert!(task.is_runnable());
        self.resched();
    }

    pub fn exit_current(&mut self, exit_code: i32) -> ! {
        let task = crate::current();
        debug!("task exit: {}, exit_code={}", task.id_name(), exit_code);
        assert!(task.is_runnable());
        assert!(!task.is_idle());
        if Arc::ptr_eq(task, &self.init_task) {
            EXITED_TASKS.lock().clear();
            axhal::misc::terminate();
        } else {
            task.set_state(TaskState::Exited);
            let task = self.scheduler.remove_task(task).expect("BUG");
            EXITED_TASKS.lock().push(task);
            WAIT_FOR_EXIT.notify_one_locked(self);
            self.resched();
        }
        unreachable!("task exited!");
    }

    pub fn block_current<F>(&mut self, wait_queue_push: F)
    where
        F: FnOnce(AxTaskRef),
    {
        let task = crate::current();
        debug!("task block: {}", task.id_name());
        assert!(task.is_runnable());
        assert!(!task.is_idle());
        task.set_state(TaskState::Blocked);
        let task = self.scheduler.remove_task(task).expect("BUG");
        wait_queue_push(task);
        self.resched();
    }

    pub fn unblock_task(&mut self, task: AxTaskRef) {
        debug!("task unblock: {}", task.id_name());
        assert!(task.is_blocked());
        task.set_state(TaskState::Runnable);
        self.scheduler.add_task(task);
    }
}

impl AxRunQueue {
    fn resched(&mut self) {
        let prev = crate::current();
        let next = self
            .scheduler
            .pick_next_task()
            .unwrap_or_else(|| self.idle_task.clone());
        if !next.is_idle() {
            self.scheduler.put_prev_task(next.clone());
        }
        self.switch_to(prev, next);
    }

    fn switch_to(&mut self, prev_task: &AxTaskRef, next_task: AxTaskRef) {
        trace!(
            "context switch: {} -> {}",
            prev_task.id_name(),
            next_task.id_name()
        );
        if Arc::ptr_eq(prev_task, &next_task) {
            return;
        }

        unsafe {
            let prev_ctx_ptr = prev_task.ctx_mut_ptr();
            let next_ctx_ptr = next_task.ctx_mut_ptr();

            // The strong reference count of `prev_task` will be decremented by 1,
            // but won't be dropped until `gc_function()` is called.
            assert!(Arc::strong_count(prev_task) > 1);
            assert!(Arc::strong_count(&next_task) > 1);

            crate::set_current(next_task);
            (*prev_ctx_ptr).switch_to(&*next_ctx_ptr);
        }
    }
}

fn gc_entry() {
    loop {
        EXITED_TASKS.lock().clear(); // drop all exited tasks and recycle resources
        WAIT_FOR_EXIT.wait();
    }
}
