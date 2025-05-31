use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::{
    pin::Pin,
    sync::atomic::AtomicU64,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use axtask::{AxTaskRef, TaskId, current, park_current_task, unpark_task};
use kspin::SpinNoIrq;

static AX_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

// unsafe fn clone(_data:*const ()) -> RawWaker {
// 	unsafe { Arc::increment_strong_count(_data as *const AxTaskRef) };
// 	RawWaker::new(_data, &AX_WAKER_VTABLE)
// }

// unsafe fn wake(_data:*const ()) {
// 	let task_ref = unsafe { Arc::from_raw(_data as *const AxTaskRef) };
// 	let id = task_ref.id().as_u64();

// 	unpark_task(id);
// }

// unsafe fn wake_by_ref(_data:*const ()) {
// 	let task_ref = unsafe { (_data as *const AxTaskRef).as_ref().unwrap() };
// 	let id = task_ref.id().as_u64();

// 	unpark_task(id);
// }

// unsafe fn drop(_data:*const ()) {
// 	unsafe { Arc::decrement_strong_count(_data as *const AxTaskRef) };
// }
//
unsafe fn clone(_data: *const ()) -> RawWaker {
    // trivial clone
    RawWaker::new(_data, &AX_WAKER_VTABLE)
}

unsafe fn wake(_data: *const ()) {
    // Call Executor pender function
    // signal_executor();
}

unsafe fn wake_by_ref(_data: *const ()) {
    // Call Executor pender function
    // signal_executor();
}

unsafe fn drop(_data: *const ()) {
    // No resource to drop
}

fn axtask_waker(task: &AxTaskRef) -> Waker {
    let data = Arc::into_raw(task.clone()) as *const ();
    let raw_waker = RawWaker::new(data, &AX_WAKER_VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

fn executor_waker() -> Waker {
    // No resource management
    let data = core::ptr::null() as *const ();
    let raw_waker = RawWaker::new(data, &AX_WAKER_VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

/// Minimal Design
pub fn async_task_run(future: impl Future<Output = ()>) {
    let curr_task = current().as_task_ref().clone();
    let waker = axtask_waker(&curr_task);
    let mut cx = Context::from_waker(&waker);
    let mut pinned = core::pin::pin!(future);

    loop {
        match pinned.as_mut().poll(&mut cx) {
            Poll::Ready(()) => {
                break;
            }
            Poll::Pending => {
                // Yield the current task to allow other tasks to run
                park_current_task();
                // Switch to the current task again
                // Repeat the loop to poll
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventId(u64);

static EVENT_ID: AtomicU64 = AtomicU64::new(0);

impl EventId {
    pub fn new() -> Self {
        EventId(EVENT_ID.fetch_add(1, core::sync::atomic::Ordering::Relaxed))
    }
}

impl From<u64> for EventId {
	fn from(value: u64) -> Self {
		EventId(value)
	}
}

type TaskWaker = (Waker, u64);
static PENDING_WAKERS: SpinNoIrq<BTreeMap<EventId, TaskWaker>> = SpinNoIrq::new(BTreeMap::new());

fn register_waker(waker: Waker, task_id: u64) {
	let id = EventId::new();
    PENDING_WAKERS.lock().insert(id, (waker, task_id));
}

fn unregister_waker(id: EventId) {
    PENDING_WAKERS.lock().remove(&id);
}

pub fn signal_event(id: EventId) {
    let mut pending_wakers = PENDING_WAKERS.lock();
    if let Some((waker, task_id)) = pending_wakers.remove(&id) {
        waker.wake_by_ref();
        unpark_task(task_id, true);
    }
}
