use alloc::{sync::Arc, vec::Vec};
use lazy_init::LazyInit;
use scheduler::BaseScheduler;
use spin::Mutex;

use crate::task::TaskState;
use crate::{AxTaskRef, Scheduler, TaskInner};

// TODO: per-CPU and IRQ-disabled lock
pub(crate) static RUN_QUEUE: LazyInit<Mutex<AxRunQueue>> = LazyInit::new();

static EXITED_TASKS: Mutex<Vec<AxTaskRef>> = Mutex::new(Vec::new());

pub(crate) struct AxRunQueue {
    idle_task: AxTaskRef,
    init_task: AxTaskRef,
    scheduler: Scheduler,
}

impl AxRunQueue {
    pub fn new() -> Self {
        let idle_task = TaskInner::new_idle();
        let init_task = TaskInner::new_init();
        let gc_task = TaskInner::new(gc_entry, "gc");
        let mut scheduler = Scheduler::new();
        scheduler.add_task(init_task.clone());
        scheduler.add_task(gc_task);
        Self {
            idle_task,
            init_task,
            scheduler,
        }
    }

    pub fn init_task(&self) -> &AxTaskRef {
        &self.init_task
    }

    pub fn add_task(&mut self, task: AxTaskRef) {
        debug!("task spawn: {}", task.id_name());
        self.scheduler.add_task(task);
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
        EXITED_TASKS.lock().push(task.clone());
        task.set_state(TaskState::Exited);
        self.resched();
        unreachable!("task exited!");
    }
}

impl AxRunQueue {
    fn resched(&mut self) {
        let prev = crate::current();
        if !prev.is_runnable() {
            self.scheduler.remove_task(prev);
        }
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
        crate::yield_now(); // TODO: wait & notify
    }
}
