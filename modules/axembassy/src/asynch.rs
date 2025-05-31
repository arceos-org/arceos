#![deny(missing_docs)]

use core::{
    cell::OnceCell,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use axtask::{park_current_task, unpark_task};
use kspin::SpinNoIrq;

pub use embassy_executor::{SendSpawner, Spawner};

pub(crate) static SPAWNER: SpinNoIrq<OnceCell<SendSpawner>> = SpinNoIrq::new(OnceCell::new());

/// Get a spawner for the system executor.
///
/// # Panics
///
/// Panics if the system executor is not initialized.
pub fn spawner() -> SendSpawner {
    let sp = SPAWNER.lock();
    *sp.get().unwrap()
}

/// Set the spawner for the system executor.
///
/// May only be called once.
pub(crate) fn set_spawner(spawner: SendSpawner) {
    let sp = SPAWNER.lock();
    let _ = sp.set(spawner);
}

fn wake(ctx: *const ()) {
    let id = ctx as u64;
    unpark_task(id, true);
}

static VTABLE: RawWakerVTable =
    RawWakerVTable::new(|ctx| RawWaker::new(ctx, &VTABLE), wake, wake, wake);

/// Blocks the current task until the given future is ready.
/// 
/// # Panics
/// 
/// Panics if not called in a thread task
pub fn block_on<F: Future>(mut fut: F) -> F::Output {
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };

    let id = axtask::current().id().as_u64();
    let raw_waker = RawWaker::new(id as *const (), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut ctx = Context::from_waker(&waker);

    loop {
        if let Poll::Ready(res) = fut.as_mut().poll(&mut ctx) {
            return res;
        }
        park_current_task();
    }
}
