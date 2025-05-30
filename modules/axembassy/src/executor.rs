use axtask::{park_current_task, unpark_task};
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::u64;
use embassy_executor::{Spawner, raw};
use log::{debug, info};

#[percpu::def_percpu]
static SIGNAL_WORK_THREAD_MODE: AtomicBool = AtomicBool::new(false);
#[percpu::def_percpu]
static EXECUTOR_TASK_ID: AtomicU64 = AtomicU64::new(u64::MAX);

#[unsafe(export_name = "__pender")]
fn __pender(_context: *mut ()) {
    SIGNAL_WORK_THREAD_MODE.with_current(|m| {
        m.store(true, Ordering::SeqCst);
    });
}

/// Unpark executor if there is a pending signal
pub fn signal_executor() {
    if unsafe {
        SIGNAL_WORK_THREAD_MODE
            .current_ref_raw()
            .load(Ordering::Acquire)
    } {
        let task_id = EXECUTOR_TASK_ID.with_current(|m| m.load(Ordering::Acquire));
        unpark_task(task_id, true);
    }
}

pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            inner: raw::Executor::new(core::ptr::null_mut()),
            not_send: PhantomData,
        }
    }

    pub fn run(&'static mut self, init: impl FnOnce(embassy_executor::Spawner)) -> ! {
        let cur_id = axtask::current().id().as_u64();
        EXECUTOR_TASK_ID.with_current(|m| {
            m.store(cur_id, Ordering::SeqCst);
        });
        init(self.inner.spawner());

        loop {
            unsafe {
                self.inner.poll();
                let to_poll = SIGNAL_WORK_THREAD_MODE.with_current(|m| m.load(Ordering::Acquire));
                if to_poll {
                    SIGNAL_WORK_THREAD_MODE.with_current(|m| {
                        m.store(false, Ordering::SeqCst);
                    });
                } else {
                    park_current_task();
                    debug!("park current task");
                }
            };
        }
    }

    pub fn spawner(&'static mut self) -> Spawner {
        self.inner.spawner()
    }
}
