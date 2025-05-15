use axtask::{park_current_task, unpark_task};
use log::debug;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::u64;
use embassy_executor::{Spawner, raw};

#[percpu::def_percpu]
static SIGNAL_WORK_THREAD_MODE: AtomicBool = AtomicBool::new(false);
#[percpu::def_percpu]
static EXECUTOR_TASK_ID:AtomicU64 = AtomicU64::new(u64::MAX);

#[unsafe(export_name = "__pender")]
fn __pender(_context: *mut ()) {
    SIGNAL_WORK_THREAD_MODE.with_current(|m| {
        m.store(true, Ordering::SeqCst);
    });
    let task_id = EXECUTOR_TASK_ID.with_current(|m| {
        m.load(Ordering::Acquire)
    });
    unpark_task(task_id, true);
}

pub fn signal_executor() {
    __pender(core::ptr::null_mut());
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
        debug!("Executor::run: {}", cur_id);
        init(self.inner.spawner());

        loop {
            unsafe {
                park_current_task();
                self.inner.poll();
                let to_poll = SIGNAL_WORK_THREAD_MODE.with_current(|m| {
                    m.load(Ordering::Acquire)
                });
                if to_poll {
                    SIGNAL_WORK_THREAD_MODE.with_current(|m| {
                        m.store(false, Ordering::SeqCst);
                    });
                } else {
                    debug!("park current task");
                }
            };
        }
    }

    pub fn spawner(&'static mut self) -> Spawner {
        self.inner.spawner()
    }
}
