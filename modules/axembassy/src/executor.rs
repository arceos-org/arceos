use core::sync::atomic::{AtomicBool, Ordering};
use core::marker::PhantomData;
use embassy_executor::raw;

static SIGNAL_WORK_THREAD_MODE: AtomicBool = AtomicBool::new(false);

#[unsafe(export_name = "__pender")]
fn __pender(_context: *mut ()) {
    SIGNAL_WORK_THREAD_MODE.store(true, Ordering::SeqCst);
}

pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl Executor {
    /// Create a new executor and initialize context with current task id
    pub fn new() -> Self {
        Self {
            inner: raw::Executor::new(core::ptr::null_mut()),
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

                if SIGNAL_WORK_THREAD_MODE.load(Ordering::SeqCst) {
                    SIGNAL_WORK_THREAD_MODE.store(false, Ordering::SeqCst);
                } else {
                    axhal::arch::wait_for_irqs();
                }
            };
        }
    }
}
