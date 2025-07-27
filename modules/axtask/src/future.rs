use alloc::{sync::Arc, task::Wake};
use core::{
    pin::{Pin, pin},
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll, Waker},
    time::Duration,
};

use axerrno::{LinuxError, LinuxResult};
use kernel_guard::NoPreemptIrqSave;
use pin_project::pin_project;

use crate::{AxTaskRef, current, current_run_queue, select_run_queue};

struct AxWaker(AxTaskRef, Arc<AtomicBool>);

impl Wake for AxWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.1.store(true, Ordering::Release);
        select_run_queue::<NoPreemptIrqSave>(&self.0).unblock_task(self.0.clone(), false);
    }
}

/// Blocks the current task until the given future is resolved.
///
/// Note that this doesn't handle interruption and is not recommended for direct
/// use in most cases.
///
/// See also [`try_block_on`].
pub fn block_on<F: IntoFuture>(fut: F) -> F::Output {
    let mut fut = pin!(fut.into_future());

    let curr = current();
    let woke = Arc::new(AtomicBool::new(false));
    let waker = Waker::from(Arc::new(AxWaker(curr.clone(), Arc::clone(&woke))));
    let mut context = Context::from_waker(&waker);
    loop {
        woke.store(false, Ordering::Release);
        match fut.as_mut().poll(&mut context) {
            Poll::Pending => {
                if !woke.load(Ordering::Acquire) {
                    let mut rq = current_run_queue::<NoPreemptIrqSave>();
                    rq.blocked_resched(|_| {});
                } else {
                    // Immediately woken
                    crate::yield_now();
                }
            }
            Poll::Ready(output) => break output,
        }
    }
}

/// Blocks the current task until the given future is resolved.
///
/// Returns:
/// - `Ok(Some(value))` if the future resolved successfully.
/// - `Err(err)` if the future resolved with an error.
/// - `Err(EINTR)` if the task was interrupted and cannot be restarted.
/// - `Ok(None)` if the task was interrupted but can be restarted.
pub fn try_block_on<F: IntoFuture<Output = LinuxResult<R>>, R>(fut: F) -> LinuxResult<Option<R>> {
    let mut fut = pin!(fut.into_future());

    let curr = current();
    let woke = Arc::new(AtomicBool::new(false));
    let waker = Waker::from(Arc::new(AxWaker(curr.clone(), Arc::clone(&woke))));
    let mut context = Context::from_waker(&waker);
    loop {
        woke.store(false, Ordering::Release);
        match fut.as_mut().poll(&mut context) {
            Poll::Pending => {
                if !woke.load(Ordering::Acquire) {
                    curr.register_interrupt_waker(context.waker());
                    let mut rq = current_run_queue::<NoPreemptIrqSave>();
                    rq.blocked_resched(|_| {});
                } else {
                    crate::yield_now();
                }
                if let Some(restart) = curr.interrupt_state() {
                    return if restart {
                        Ok(None)
                    } else {
                        Err(LinuxError::EINTR)
                    };
                }
            }
            Poll::Ready(output) => return output.map(Some),
        }
    }
}

/// No restart version of [`try_block_on`].
///
/// The difference is that this function will always return `Err(EINTR)` when
/// interrupted, no matter whether the signal says the task can be restarted or
/// not.
pub fn try_block_on_no_restart<F: IntoFuture<Output = LinuxResult<R>>, R>(
    fut: F,
) -> LinuxResult<R> {
    try_block_on(fut).and_then(|opt| opt.ok_or(LinuxError::EINTR))
}

/// Waits until `duration` has elapsed.
pub fn sleep(duration: Duration) -> Sleep {
    sleep_until(axhal::time::wall_time() + duration)
}

/// Waits until `deadline` is reached.
pub fn sleep_until(deadline: Duration) -> Sleep {
    if deadline > axhal::time::wall_time() {
        let curr = current();
        crate::timers::set_alarm_wakeup(deadline, curr.clone());
    }

    Sleep { deadline }
}

/// Future returned by `sleep` and `sleep_until`.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Sleep {
    deadline: Duration,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if axhal::time::wall_time() < self.deadline {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

/// Requires a `Future` to complete before the specified duration has elapsed.
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

/// Requires a `Future` to complete before the optional duration has elapsed.
pub fn timeout_opt<F: IntoFuture>(fut: F, duration: Option<Duration>) -> Timeout<F::IntoFuture> {
    Timeout {
        inner: fut.into_future(),
        delay: duration.map_or_else(
            || Sleep {
                deadline: Duration::MAX,
            },
            sleep,
        ),
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
