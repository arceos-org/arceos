use core::{pin::Pin, task::{Context, Poll, RawWaker, RawWakerVTable, Waker}};
use alloc::sync::Arc;

// use core::task::RawWaker;

use axtask::{current, park_current_task, unpark_task, AxTaskRef};
// 
use crate::executor::__pender;

static AX_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
	clone,
	wake,
	wake_by_ref,
	drop,
); 

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
unsafe fn clone(_data:*const ()) -> RawWaker {
	// trivial clone
	RawWaker::new(_data, &AX_WAKER_VTABLE)
}

unsafe fn wake(_data: *const ()) {
	// Call Executor pender function
	__pender(core::ptr::null_mut());
}

unsafe fn wake_by_ref(_data: *const ()) {
	// Call Executor pender function
	__pender(core::ptr::null_mut());
}

unsafe fn drop(_data: *const ()) {
	// No resource to drop
}

fn axtask_waker(task:&AxTaskRef) -> Waker {
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
pub fn async_task_run(future:impl Future<Output = ()>) {
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