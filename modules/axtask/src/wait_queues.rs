use core::{sync::atomic::AtomicU32, time::Duration};

use alloc::collections::BTreeMap;

use kspin::SpinNoIrq;

use crate::WaitQueue;

pub type Futex = AtomicU32;

static FUTEX_WAIT_QUEUES: SpinNoIrq<BTreeMap<usize, WaitQueue>> = SpinNoIrq::new(BTreeMap::new());

pub fn futex_wake(futex: &Futex) {
    let futex_addr = futex as *const _ as usize;
    let mut wait_queues = FUTEX_WAIT_QUEUES.lock();
    if let Some(queue) = wait_queues.get_mut(&futex_addr) {
        // Wake up one task waiting on this queue.
        // `notify_one(true)` means it will potentially yield the CPU
        // if the woken task has higher priority.
        queue.notify_one(true);
    }
    // The lock is automatically released when wait_queues goes out of scope
}

pub fn futex_wake_all(futex: &Futex) {
    let futex_addr = futex as *const _ as usize;
    let mut wait_queues = FUTEX_WAIT_QUEUES.lock();
    if let Some(queue) = wait_queues.get_mut(&futex_addr) {
        // Wake up all tasks waiting on this queue.
        queue.notify_all(true);
    }
    // The lock is automatically released when wait_queues goes out of scope
}

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
                panic!("wait_timeout is not supported in without irq feature");
            }
        }
        None => {
            queue.wait();
            true
        }
    };
    waited
}