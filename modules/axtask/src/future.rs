use alloc::{sync::Arc, task::Wake};
use core::{
    pin::{Pin, pin},
    task::{Context, Poll, Waker},
    time::Duration,
};

use kernel_guard::NoPreemptIrqSave;
use pin_project::pin_project;

use crate::{AxTaskRef, current, current_run_queue, select_run_queue};

pub(crate) struct AxWaker(AxTaskRef);

impl Wake for AxWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        select_run_queue::<NoPreemptIrqSave>(&self.0).unblock_task(self.0.clone(), true);
    }
}

/// Blocks the current task until the given future is resolved.
pub fn block_on<F: IntoFuture>(fut: F) -> F::Output {
    let mut fut = pin!(fut.into_future());

    let curr = current();
    let waker = Waker::from(Arc::new(AxWaker(curr.clone())));
    let mut context = Context::from_waker(&waker);
    loop {
        match fut.as_mut().poll(&mut context) {
            Poll::Pending => {
                let mut rq = current_run_queue::<NoPreemptIrqSave>();
                rq.blocked_resched(|_| {});
            }
            Poll::Ready(output) => break output,
        }
    }
}

/// Waits until `duration` has elapsed.
pub fn sleep(duration: Duration) -> Sleep {
    sleep_until(axhal::time::wall_time() + duration)
}

/// Waits until `deadline` is reached.
pub fn sleep_until(deadline: Duration) -> Sleep {
    if deadline <= axhal::time::wall_time() {
        return Sleep { polled: true };
    }

    let curr = current();
    crate::timers::set_alarm_wakeup(deadline, curr.clone());
    Sleep { polled: false }
}

/// Future returned by `sleep` and `sleep_until`.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Sleep {
    polled: bool,
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.polled {
            Poll::Ready(())
        } else {
            self.polled = true;
            Poll::Pending
        }
    }
}

/// Requires a Future to complete before the specified duration has elapsed.
pub fn timeout<F: IntoFuture>(fut: F, duration: Duration) -> Timeout<F::IntoFuture> {
    Timeout {
        inner: fut.into_future(),
        delay: sleep(duration),
    }
}

/// Requires a `Future` to complete before the specified instant in time.
pub fn timeout_at<F: IntoFuture>(fut: F, deadline: Duration) -> Timeout<F::IntoFuture> {
    Timeout {
        inner: fut.into_future(),
        delay: sleep_until(deadline),
    }
}

/// Future returned by `timeout` and `timeout_at`.
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[pin_project]
pub struct Timeout<F> {
    #[pin]
    inner: F,
    #[pin]
    delay: Sleep,
}

impl<F> Timeout<F> {
    pub fn into_inner(self) -> F {
        self.inner
    }
}

impl<F: Future> Future for Timeout<F> {
    type Output = Option<F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.inner.poll(cx) {
            Poll::Ready(output) => Poll::Ready(Some(output)),
            Poll::Pending => {
                if this.delay.poll(cx).is_ready() {
                    Poll::Ready(None)
                } else {
                    Poll::Pending
                }
            }
        }
    }
}
