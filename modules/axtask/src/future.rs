use alloc::{sync::Arc, task::Wake};
use core::{
    fmt,
    future::poll_fn,
    pin::{Pin, pin},
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll, Waker},
    time::Duration,
};

use axerrno::{AxError, AxResult};
use axhal::time::TimeValue;
use axio::{IoEvents, Pollable};
use futures::FutureExt;
use kernel_guard::NoPreemptIrqSave;
use pin_project::pin_project;

use crate::{
    AxTaskRef, WeakAxTaskRef, current, current_run_queue, select_run_queue,
    timers::{TimerKey, cancel_timer, has_timer, set_timer},
};

struct AxWaker {
    task: WeakAxTaskRef,
    woke: AtomicBool,
}

impl Wake for AxWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if let Some(task) = self.task.upgrade() {
            self.woke.store(true, Ordering::Release);
            select_run_queue::<NoPreemptIrqSave>(&task).unblock_task(task, false);
        }
    }
}

/// Blocks the current task until the given future is resolved.
///
/// Note that this doesn't handle interruption and is not recommended for direct
/// use in most cases.
///
/// See also [`try_block_on`].
pub fn block_on<F: IntoFuture>(f: F) -> F::Output {
    let mut fut = pin!(f.into_future());

    let curr = current();
    // It's necessary to keep a strong reference to the current task
    // to prevent it from being dropped while blocking.
    let task = curr.clone();

    let waker = Arc::new(AxWaker {
        task: Arc::downgrade(&task),
        woke: AtomicBool::new(false),
    });
    let woke = &waker.woke;
    let waker = Waker::from(waker.clone());
    let mut cx = Context::from_waker(&waker);

    loop {
        woke.store(false, Ordering::Release);
        match fut.as_mut().poll(&mut cx) {
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

/// Future returned by [`interruptible`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[pin_project]
pub struct Interruptible<F> {
    #[pin]
    inner: F,
    task: AxTaskRef,
}

impl<F> Future for Interruptible<F>
where
    F: Future,
{
    type Output = Result<F::Output, Interrupted>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if this.task.interrupted() {
            return Poll::Ready(Err(Interrupted));
        } else {
            this.task.on_interrupt(cx.waker());
        }

        this.inner.poll(cx).map(Ok)
    }
}

/// Makes a future interruptible.
pub fn interruptible<F: IntoFuture>(f: F) -> Interruptible<F::IntoFuture> {
    Interruptible {
        inner: f.into_future(),
        task: current().clone(),
    }
}

/// Future returned by `sleep` and `sleep_until`.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct TimerFuture(Option<TimerKey>);

impl TimerFuture {
    pub fn new(deadline: Option<Duration>) -> Self {
        if let Some(deadline) = deadline {
            Self(set_timer(deadline, &current()))
        } else {
            return Self(None);
        }
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(key) = self.0
            && !has_timer(&key)
        {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl Drop for TimerFuture {
    fn drop(&mut self) {
        if let Some(key) = self.0.take() {
            cancel_timer(&key);
        }
    }
}

/// Waits until `duration` has elapsed.
pub fn sleep(duration: Duration) -> TimerFuture {
    sleep_until(axhal::time::wall_time() + duration)
}

/// Waits until `deadline` is reached.
pub fn sleep_until(deadline: TimeValue) -> TimerFuture {
    TimerFuture::new(Some(deadline))
}

/// Error returned by [`timeout`] and [`timeout_at`].
#[derive(Debug, PartialEq, Eq)]
pub struct Elapsed;

impl fmt::Display for Elapsed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "deadline elapsed")
    }
}

impl core::error::Error for Elapsed {}

impl From<Elapsed> for AxError {
    fn from(_: Elapsed) -> Self {
        AxError::TimedOut
    }
}

/// Future returned by [`timeout`] and [`timeout_at`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[pin_project]
pub struct Timeout<F> {
    #[pin]
    inner: F,
    delay: TimerFuture,
}

impl<F: Future> Future for Timeout<F> {
    type Output = Result<F::Output, Elapsed>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if let Poll::Ready(()) = this.delay.poll_unpin(cx) {
            return Poll::Ready(Err(Elapsed));
        }

        this.inner.poll(cx).map(Ok)
    }
}

pub fn timeout<F: IntoFuture>(duration: Option<Duration>, f: F) -> Timeout<F::IntoFuture> {
    timeout_at(
        duration.and_then(|x| x.checked_add(axhal::time::wall_time())),
        f,
    )
}

pub fn timeout_at<F: IntoFuture>(deadline: Option<TimeValue>, f: F) -> Timeout<F::IntoFuture> {
    Timeout {
        inner: f.into_future(),
        delay: TimerFuture::new(deadline),
    }
}

pub struct Poller<'a, P> {
    pollable: &'a P,
    events: IoEvents,
    non_blocking: bool,
    timeout: Option<Duration>,
}

impl<'a, P: Pollable> Poller<'a, P> {
    pub fn new(pollable: &'a P, events: IoEvents) -> Self {
        Poller {
            pollable,
            events,
            non_blocking: false,
            timeout: None,
        }
    }

    pub fn non_blocking(mut self, non_blocking: bool) -> Self {
        self.non_blocking = non_blocking;
        self
    }

    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn poll<T>(self, mut f: impl FnMut() -> AxResult<T>) -> AxResult<T> {
        if self.timeout.is_some_and(|it| it.as_micros() == 0) {
            return match f() {
                Ok(value) => Ok(value),
                Err(AxError::WouldBlock) => {
                    if self.non_blocking {
                        Err(AxError::WouldBlock)
                    } else {
                        Err(AxError::TimedOut)
                    }
                }
                Err(e) => Err(e),
            };
        }

        let fut = poll_fn(move |cx| match f() {
            Ok(value) => Poll::Ready(Ok(value)),
            Err(AxError::WouldBlock) => {
                if self.non_blocking {
                    return Poll::Ready(Err(AxError::WouldBlock));
                }
                self.pollable.register(cx, self.events);
                match f() {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(AxError::WouldBlock) => Poll::Pending,
                    Err(e) => Poll::Ready(Err(e)),
                }
            }
            Err(e) => Poll::Ready(Err(e)),
        });

        block_on(interruptible(timeout(self.timeout, fut)))??
    }
}
