use core::{sync::atomic::AtomicU64, task::Waker};

use alloc::collections::BTreeMap;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use kspin::SpinNoIrq;
use log::info;
use static_cell::StaticCell;

use crate::executor::{self, signal_executor};

static EXECUTOR: StaticCell<executor::Executor> = StaticCell::new();

pub fn runtime<F>(initial: F) -> !
where
    F: FnOnce(Spawner) + Send + 'static,
{
    let exec = EXECUTOR.init(executor::Executor::new());
    exec.run(initial)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventId(u64);

static EVENT_ID: AtomicU64 = AtomicU64::new(0);

impl EventId {
    pub fn new() -> Self {
        EventId(EVENT_ID.fetch_add(1, core::sync::atomic::Ordering::Relaxed))
    }
}

static PENDING_WAKERS: SpinNoIrq<BTreeMap<EventId, Waker>> = SpinNoIrq::new(BTreeMap::new());

fn register_waker(id: EventId, waker: Waker) {
    PENDING_WAKERS.lock().insert(id, waker);
}

fn unregister_waker(id: EventId) {
    PENDING_WAKERS.lock().remove(&id);
}

pub fn signal_event(id: EventId) {
    let mut pending_wakers = PENDING_WAKERS.lock();
    if let Some(waker) = pending_wakers.remove(&id) {
        waker.wake_by_ref();
        signal_executor();
    }
}

#[embassy_executor::task]
async fn tick(_sec: u64) {
    for _ in 0..4 {
        info!("tick for {} sec", _sec);
        yield_now().await;
    }
}

#[embassy_executor::task]
async fn tick_2(_sec: u64) {
    for _ in 0..4 {
        info!("tick for {} sec", _sec);
        yield_now().await;
    }
}

pub fn init() {
    let exec = EXECUTOR.init(executor::Executor::new());
    exec.run(|s| {
        s.spawn(tick(1)).unwrap();
        s.spawn(tick_2(2)).unwrap();
    })
}
