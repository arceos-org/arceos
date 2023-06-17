//
// Task & WaitQueue.
//

use alloc::boxed::Box;
use alloc::string::ToString;
use axtask::{AxTaskRef, WaitQueue};
use core::sync::atomic::{AtomicU32, Ordering};
use core::time::Duration;

static WQ: WaitQueue = WaitQueue::new();

#[no_mangle]
pub fn sys_futex_wait(futex: &AtomicU32, expected: u32, timeout: Option<Duration>) -> bool {
    let condition = || {
        futex
            .compare_exchange(expected, expected, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
    };

    match timeout {
        #[allow(unused_variables)]
        Some(duration) => {
            #[cfg(not(feature = "irq"))]
            panic!("Need to enable 'irq' feature.");
            #[cfg(feature = "irq")]
            !WQ.wait_timeout_until(duration, condition)
        }
        None => {
            WQ.wait_until(condition);
            true
        }
    }
}

#[no_mangle]
pub fn sys_futex_wake(_futex: &AtomicU32, count: i32) {
    if count == i32::MAX {
        WQ.notify_all(false);
    } else {
        for _ in 0..count {
            WQ.notify_one(false);
        }
    }
}
