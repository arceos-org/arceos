use alloc::{sync::Arc, vec::Vec};
use lazy_init::LazyInit;
use scheduler::BaseScheduler;
use spin::Mutex;

use crate::task::TaskState;
use crate::{AxTask, SchedulerImpl};

// TODO: per-CPU and IRQ-disabled lock
pub(crate) static RUN_QUEUE: LazyInit<Mutex<AxRunQueue>> = LazyInit::new();

static EXITED_TASKS: Mutex<Vec<Arc<AxTask>>> = Mutex::new(Vec::new());

pub(crate) struct AxRunQueue {
    idle_task: Arc<AxTask>,
    init_task: Arc<AxTask>,
    scheduler: SchedulerImpl<AxTask>,
}

impl AxRunQueue {
    pub fn new() -> Self {
        let idle_task = AxTask::new_idle();
        let init_task = AxTask::new_init();
        let gc_task = AxTask::new(gc_function, "gc");
        let mut scheduler = SchedulerImpl::new();
        scheduler.add_task(init_task.clone());
        scheduler.add_task(gc_task);
        Self {
            idle_task,
            init_task,
            scheduler,
        }
    }

    pub fn init_task(&self) -> &Arc<AxTask> {
        &self.init_task
    }

    pub fn add_task(&mut self, task: Arc<AxTask>) {
        debug!("task spawn: {}", task.id_name());
        self.scheduler.add_task(task);
    }

    pub fn yield_current(&mut self) {
        let task = crate::current();
        debug!("task yield: {}", task.id_name());
        assert!(task.is_runnable());
        assert!(!task.is_idle());
        self.scheduler.yield_task(task.clone());
        self.resched();
    }

    pub fn exit_current(&mut self, exit_code: i32) -> ! {
        let task = crate::current();
        debug!("task exit: {}, exit_code={}", task.id_name(), exit_code);
        assert!(task.is_runnable());
        assert!(!task.is_idle());
        EXITED_TASKS.lock().push(task.clone());
        task.set_state(TaskState::Exited);
        self.resched();
        unreachable!("task exited!");
    }

    pub fn resched(&mut self) {
        let prev = crate::current();
        if !prev.is_runnable() {
            self.scheduler.remove_task(prev);
        }
        let next = self
            .scheduler
            .pick_next_task(prev)
            .unwrap_or(&self.idle_task)
            .clone();
        self.switch_to(prev, next);
    }
}

impl AxRunQueue {
    fn switch_to(&self, prev_task: &Arc<AxTask>, next_task: Arc<AxTask>) {
        trace!(
            "context switch: {} -> {}",
            prev_task.id_name(),
            next_task.id_name()
        );
        if Arc::ptr_eq(prev_task, &next_task) {
            return;
        }

        let prev_ctx_ptr = prev_task.ctx_mut_ptr();
        let next_ctx_ptr = next_task.ctx_mut_ptr();

        // The strong reference count of `prev_task` will be decremented by 1,
        // but won't be dropped until `gc_function()` is called.
        assert!(Arc::strong_count(prev_task) > 1);
        assert!(Arc::strong_count(&next_task) > 1);

        unsafe {
            crate::set_current(next_task);
            (*prev_ctx_ptr).switch_to(&*next_ctx_ptr);
        }
    }
}

fn gc_function() {
    loop {
        EXITED_TASKS.lock().clear(); // drop all exited tasks and recycle resources
        crate::yield_now(); // TODO: wait & notify
    }
}
