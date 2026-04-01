//! Future support.

use alloc::{sync::Arc, task::Wake};
use core::{
    fmt,
    future::poll_fn,
    pin::pin,
    task::{Context, Poll, Waker},
};

use axerrno::AxError;
use kernel_guard::NoPreemptIrqSave;
use kspin::SpinNoIrq;

use crate::{AxTaskRef, WeakAxTaskRef, current, current_run_queue, select_run_queue};

mod poll;
pub use poll::*;

mod time;
pub use time::*;

struct AxWaker {
    task: WeakAxTaskRef,
    woke: SpinNoIrq<bool>,
}

impl AxWaker {
    fn new(task: &AxTaskRef) -> Arc<Self> {
        Arc::new(AxWaker {
            task: Arc::downgrade(task),
            woke: SpinNoIrq::new(false),
        })
    }
}

impl Wake for AxWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if let Some(task) = self.task.upgrade() {
            let mut rq = select_run_queue::<NoPreemptIrqSave>(&task);
            *self.woke.lock() = true;
            rq.unblock_task(task, false);
        }
    }
}

/// Blocks the current task until the given future is resolved.
///
/// Note that this doesn't handle interruption and is not recommended for direct
/// use in most cases.
pub fn block_on<F: IntoFuture>(f: F) -> F::Output {
    let mut fut = pin!(f.into_future());

    let curr = current();
    // It's necessary to keep a strong reference to the current task
    // to prevent it from being dropped while blocking.
    let task = curr.clone();

    let axwaker = AxWaker::new(&task);
    let waker = Waker::from(axwaker.clone());
    let mut cx = Context::from_waker(&waker);

    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Pending => {
                let mut rq = current_run_queue::<NoPreemptIrqSave>();
                let mut woke = axwaker.woke.lock();
                if !*woke {
                    // blocked_resched() will set *woke = false and drop
                    // the guard internally before rescheduling. When this
                    // task is woken, woke will be set to true by the waker
                    // and we'll re-enter the loop to poll again.
                    rq.future_blocked_resched(woke);
                } else {
                    // ③ woke = true: waker fired between poll() returning
                    // Pending and us acquiring the lock.
                    // Clear under the current lock hold, then drop.
                    // If an interrupt sets woke=true after drop(), the next
                    // loop iteration will see it correctly.
                    *woke = false;
                    drop(woke);
                    crate::yield_now();
                }
            }
            Poll::Ready(output) => break output,
        }
    }
}

/// Error returned by [`interruptible`].
#[derive(Debug, PartialEq, Eq)]
pub struct Interrupted;

impl fmt::Display for Interrupted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "interrupted")
    }
}

impl core::error::Error for Interrupted {}

impl From<Interrupted> for AxError {
    fn from(_: Interrupted) -> Self {
        AxError::Interrupted
    }
}

/// Makes a future interruptible.
pub async fn interruptible<F: IntoFuture>(f: F) -> Result<F::Output, Interrupted> {
    let mut f = pin!(f.into_future());
    let curr = current();
    poll_fn(|cx| {
        if curr.poll_interrupt(cx).is_ready() {
            return Poll::Ready(Err(Interrupted));
        }
        f.as_mut().poll(cx).map(Ok)
    })
    .await
}
