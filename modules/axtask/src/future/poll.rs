use core::{future::poll_fn, task::Poll};

use axerrno::{AxError, AxResult};
use axpoll::{IoEvents, Pollable};

/// A helper to wrap a synchronous non-blocking I/O function into an
/// asynchronous function.
///
/// # Arguments
///
/// * `pollable`: The pollable object to register for I/O events.
/// * `events`: The I/O events to wait for.
/// * `non_blocking`: If true, the function will return `AxError::WouldBlock`
///   immediately when the I/O operation would block.
/// * `f`: The synchronous non-blocking I/O function to be wrapped. It should
///   return `AxError::WouldBlock` when the operation would block.
pub async fn poll_io<P: Pollable, F: FnMut() -> AxResult<T>, T>(
    pollable: &P,
    events: IoEvents,
    non_blocking: bool,
    mut f: F,
) -> AxResult<T> {
    super::interruptible(poll_fn(move |cx| match f() {
        Ok(value) => Poll::Ready(Ok(value)),
        Err(AxError::WouldBlock) => {
            if non_blocking {
                return Poll::Ready(Err(AxError::WouldBlock));
            }
            pollable.register(cx, events);
            match f() {
                Ok(value) => Poll::Ready(Ok(value)),
                Err(AxError::WouldBlock) => Poll::Pending,
                Err(e) => Poll::Ready(Err(e)),
            }
        }
        Err(e) => Poll::Ready(Err(e)),
    }))
    .await?
}

#[cfg(feature = "irq")]
/// Registers a waker for the given IRQ number.
pub fn register_irq_waker(irq: usize, waker: &core::task::Waker) {
    use alloc::collections::BTreeMap;

    use axpoll::PollSet;
    use kspin::SpinNoIrq;

    static POLL_IRQ: SpinNoIrq<BTreeMap<usize, PollSet>> = SpinNoIrq::new(BTreeMap::new());

    fn irq_hook(irq: usize) {
        if let Some(s) = POLL_IRQ.lock().get(&irq) {
            s.wake();
        }
    }
    axhal::irq::register_irq_hook(irq_hook);

    POLL_IRQ.lock().entry(irq).or_default().register(waker);

    // With MSI-X (edge-triggered), enabling the IRQ here is safe: an
    // edge-triggered interrupt fires once per assertion and does not re-fire
    // while the line is held. There is no risk of a spurious wakeup loop
    // during the poll phase.
    axhal::irq::set_enable(irq, true);
}
