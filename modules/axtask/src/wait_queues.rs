use core::{sync::atomic::AtomicU32, time::Duration};

use alloc::collections::BTreeMap;

use kspin::SpinNoIrq;

use crate::WaitQueue;

/// The address of a futex
pub type Futex = AtomicU32;

/// A global map that associates the memory address of a `Futex` instance
/// with a `WaitQueue`.
///
/// When tasks need to wait on a specific `Futex`, they register themselves
/// with the `WaitQueue` corresponding to that `Futex`'s memory address.
static FUTEX_WAIT_QUEUES: SpinNoIrq<BTreeMap<usize, WaitQueue>> = SpinNoIrq::new(BTreeMap::new());

/// Wakes up a single task that is currently waiting on the given `Futex`.
pub fn futex_wake(futex: &Futex) {
    let futex_addr = futex as *const _ as usize;
    let mut wait_queues = FUTEX_WAIT_QUEUES.lock();
    if let Some(queue) = wait_queues.get_mut(&futex_addr) {
        // Wake up one task waiting on this queue.
        // `notify_one(true)` means it will potentially yield the CPU
        // if the woken task has higher priority.
        queue.notify_one(true);
    }
}

/// Wakes up all tasks that are currently waiting on the given `Futex`.
pub fn futex_wake_all(futex: &Futex) {
    let futex_addr = futex as *const _ as usize;
    let mut wait_queues = FUTEX_WAIT_QUEUES.lock();
    if let Some(queue) = wait_queues.get_mut(&futex_addr) {
        // Wake up all tasks waiting on this queue.
        queue.notify_all(true);
    }
}

/// Attempts to wait on a `Futex` until its value is no longer `expected`.
pub fn futex_wait(futex: &Futex, expected: u32, timeout: Option<Duration>) -> bool {
    let futex_addr = futex as *const _ as usize;

    let current_val = futex.load(core::sync::atomic::Ordering::Relaxed);
    if current_val != expected {
        return false;
    }

    let mut wait_queues = FUTEX_WAIT_QUEUES.lock();
    let queue = wait_queues
        .entry(futex_addr)
        .or_insert_with(|| WaitQueue::new());

    let waited = match timeout {
        Some(dur) => {
            #[cfg(feature = "irq")]
            {
                !queue.wait_timeout(dur)
            }
            #[cfg(not(feature = "irq"))]
            {
                panic!("wait_timeout is not supported without irq feature");
            }
        }
        None => {
            queue.wait();
            true
        }
    };
    waited
}
