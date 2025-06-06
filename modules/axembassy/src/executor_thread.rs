use axtask::{park_current_task, unpark_task};
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::u64;
use embassy_executor::raw;

#[percpu::def_percpu]
static SIGNAL_WORK_THREAD_MODE: AtomicBool = AtomicBool::new(false);

#[unsafe(export_name = "__pender")]
fn __pender(_context: *mut ()) {
    SIGNAL_WORK_THREAD_MODE.with_current(|m| {
        m.store(true, Ordering::SeqCst);
    });
    let id = _context as u64;
    unpark_task(id, true);
}

pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl Executor {
    /// Create a new executor and initialize context with current task id
    pub fn new() -> Self {
        let cur_id = axtask::current().id().as_u64();
        Self {
            inner: raw::Executor::new(cur_id as *mut ()),
            not_send: PhantomData,
        }
    }

    /// Runs the executor.
    ///
    /// The `init` closure is called with a [`Spawner`] that spawns tasks on
    /// this executor. Use it to spawn the initial task(s). After `init` returns,
    /// the executor starts running the tasks.
    ///
    pub fn run(&'static mut self, init: impl FnOnce(embassy_executor::Spawner)) -> ! {
        init(self.inner.spawner());

        loop {
            unsafe {
                self.inner.poll();
                let polled = SIGNAL_WORK_THREAD_MODE.with_current(|m| m.load(Ordering::Acquire));
                if polled {
                    SIGNAL_WORK_THREAD_MODE.with_current(|m| {
                        m.store(false, Ordering::SeqCst);
                    });
                } else {
                    park_current_task();
                }
            };
        }
    }
}
