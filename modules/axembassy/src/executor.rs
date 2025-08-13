use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use embassy_executor::raw;

#[cfg(feature = "executor-single")]
static SIGNAL_SINGLE: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "executor-thread")]
#[percpu::def_percpu]
static SINGAL_THREAD: AtomicBool = AtomicBool::new(false);

#[unsafe(export_name = "__pender")]
fn __pender(_context: *mut ()) {
    #[cfg(feature = "executor-single")]
    SIGNAL_SINGLE.store(true, Ordering::SeqCst);

    #[cfg(feature = "executor-thread")]
    SINGAL_THREAD.with_current(|m| {
        m.store(true, Ordering::SeqCst);
    });
}

/// An executor based on the [embassy_executor](https://docs.rs/embassy-executor/latest/embassy_executor/) crate
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
    /// The `init` closure is called with a [`embassy_executor::Spawner`] that spawns tasks on
    /// this executor. Use it to spawn the initial task(s). After `init` returns,
    /// the executor starts running the tasks.
    ///
    pub fn run(&'static mut self, init: impl FnOnce(embassy_executor::Spawner)) -> ! {
        init(self.inner.spawner());

        loop {
            unsafe {
                self.inner.poll();

                #[cfg(feature = "executor-single")]
                {
                    if SIGNAL_SINGLE.load(Ordering::SeqCst) {
                        SIGNAL_SINGLE.store(false, Ordering::SeqCst);
                    } else {
                        axhal::asm::wait_for_irqs();
                    }
                }

                #[cfg(feature = "executor-thread")]
                {
                    let polled = SINGAL_THREAD.with_current(|m| m.load(Ordering::Acquire));
                    if polled {
                        SINGAL_THREAD.with_current(|m| {
                            m.store(false, Ordering::Release);
                        });
                    } else {
                        // park_current_task();
                        axtask::yield_now();
                    }
                }
            };
        }
    }
}
