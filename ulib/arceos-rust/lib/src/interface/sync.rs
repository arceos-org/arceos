use crate::err;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use arceos_api::modules::axlog::{info, trace};
use arceos_api::modules::axsync::Mutex;
use arceos_api::modules::axtask::WaitQueue;
use arceos_api::task::ax_yield_now;
use arceos_posix_api::ctypes::timespec;
use axerrno::LinuxError;
use core::time::Duration;

static FUTEX_TABLE: Mutex<BTreeMap<usize, Arc<WaitQueue>>> = Mutex::new(BTreeMap::new());

/// If the value at address matches the expected value, park the current thread until it is either
/// woken up with [`futex_wake`] (returns 0) or an optional timeout elapses (returns -ETIMEDOUT).
///
/// Setting `timeout` to null means the function will only return if [`futex_wake`] is called.
/// Otherwise, `timeout` is interpreted as an absolute time measured with [`CLOCK_MONOTONIC`].
/// If [`FUTEX_RELATIVE_TIMEOUT`] is set in `flags` the timeout is understood to be relative
/// to the current time.
///
/// Returns -EINVAL if `address` is null, the timeout is negative or `flags` contains unknown values.
#[unsafe(no_mangle)]
pub fn sys_futex_wait(
    address: *mut u32,
    expected: u32,
    timeout: *const timespec,
    _flags: u32,
) -> i32 {
    let Some(value) = (unsafe { address.as_ref() }) else {
        return err(LinuxError::EINVAL);
    };
    if *value != expected {
        return err(LinuxError::EAGAIN);
    }
    let wait_queue = {
        let mut table = FUTEX_TABLE.lock();
        table
            .entry(address as usize)
            .or_insert_with(|| Arc::new(WaitQueue::new()))
            .clone()
    };
    trace!(
        "futex wait on address {:p} with expected value {}",
        address, expected
    );
    if let Some(timeout) = unsafe { timeout.as_ref() } {
        trace!("called sys_futex_wait with timeout: {:?}", timeout);
        let duration = Duration::new(timeout.tv_sec as u64, timeout.tv_nsec as u32);
        wait_queue.wait_timeout(duration);
    } else {
        trace!("called sys_futex_wait without timeout");
        wait_queue.wait();
    }
    0
}

/// Wake `count` threads waiting on the futex at `address`. Returns the number of threads
/// woken up (saturates to `i32::MAX`). If `count` is `i32::MAX`, wake up all matching
/// waiting threads. If `count` is negative or `address` is null, returns -EINVAL.
#[unsafe(no_mangle)]
pub fn sys_futex_wake(address: *mut u32, count: i32) -> i32 {
    info!(
        "called sys_futex_wake with address {:p} and count {}",
        address, count
    );
    if count < 0 {
        return err(LinuxError::EINVAL);
    }
    let wait_queue = {
        let table = FUTEX_TABLE.lock();
        match table.get(&(address as usize)) {
            Some(queue) => queue.clone(),
            None => return 0,
        }
    };
    for woken_count in 0..count {
        if !wait_queue.notify_one(true) {
            trace!("futex woke {} threads", woken_count);
            return woken_count;
        }
    }
    trace!("futex woke {} threads", count);
    ax_yield_now();
    count
}
