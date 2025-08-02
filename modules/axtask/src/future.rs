use alloc::{sync::Arc, task::Wake};
use core::{
    future::poll_fn,
    pin::{Pin, pin},
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll, Waker},
    time::Duration,
};

use axerrno::{LinuxError, LinuxResult};
use axio::{IoEvents, Pollable};
use futures::FutureExt;
use kernel_guard::NoPreemptIrqSave;
use pin_project::pin_project;

use crate::{WeakAxTaskRef, current, current_run_queue, select_run_queue};

struct AxWaker(WeakAxTaskRef, Arc<AtomicBool>);

impl Wake for AxWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if let Some(task) = self.0.upgrade() {
            self.1.store(true, Ordering::Release);
            select_run_queue::<NoPreemptIrqSave>(&task).unblock_task(task.clone(), false);
        }
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
    // It's necessary to keep a strong reference to the current task
    // to prevent it from being dropped while blocking.
    let task = curr.clone();
    let woke = Arc::new(AtomicBool::new(false));
    let waker = Waker::from(Arc::new(AxWaker(Arc::downgrade(&task), Arc::clone(&woke))));
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
    let task = curr.clone();
    let woke = Arc::new(AtomicBool::new(false));
    let waker = Waker::from(Arc::new(AxWaker(Arc::downgrade(&task), Arc::clone(&woke))));
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
pub fn block_on_interruptible<F: IntoFuture<Output = LinuxResult<R>>, R>(fut: F) -> LinuxResult<R> {
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
        crate::timers::set_alarm_wakeup(deadline, curr.as_task_ref());
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

/// Requires a `Future` to complete before the optional instant in time.
pub fn timeout_at_opt<F: IntoFuture>(fut: F, deadline: Option<Duration>) -> Timeout<F::IntoFuture> {
    Timeout {
        inner: fut.into_future(),
        delay: deadline.map_or_else(
            || Sleep {
                deadline: Duration::MAX,
            },
            sleep_until,
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

    pub fn poll<T>(self, mut f: impl FnMut() -> LinuxResult<T>) -> LinuxResult<T> {
        block_on_interruptible(
            timeout_opt(
                poll_fn(move |cx| match f() {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(LinuxError::EAGAIN) => {
                        if self.non_blocking {
                            return Poll::Ready(Err(LinuxError::EAGAIN));
                        }
                        self.pollable.register(cx, self.events);
                        match f() {
                            Ok(value) => Poll::Ready(Ok(value)),
                            Err(LinuxError::EAGAIN) => Poll::Pending,
                            Err(e) => Poll::Ready(Err(e)),
                        }
                    }
                    Err(e) => Poll::Ready(Err(e)),
                }),
                self.timeout,
            )
            .map(|opt| opt.ok_or(LinuxError::EINTR)?),
        )
    }
}
