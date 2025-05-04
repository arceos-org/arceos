extern crate alloc;

use crate::MutexGuard;
use axtask::{Futex, futex_wait, futex_wake, futex_wake_all};

use core::{sync::atomic::AtomicU32, time::Duration};

pub struct Condvar {
    // The value of this atomic is simply incremented on every notification.
    // This is used by `.wait()` to not miss any notifications after
    // unlocking the mutex and before waiting for notifications.
    futex: Futex,
}

impl Condvar {
    #[inline]
    pub const fn new() -> Self {
        Self {
            futex: AtomicU32::new(0),
        }
    }

    pub fn notify_one(&self) {
        self.futex
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        futex_wake(&self.futex);
    }

    pub fn notify_all(&self) {
        self.futex
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        futex_wake_all(&self.futex);
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        // wait with no timeout should always return the guard
        self.wait_optional_timeout(guard, None)
            .expect("Condvar::wait with no timeout should not return None on timeout")
    }

    pub fn wait_timeout<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        timeout: Duration,
    ) -> Option<MutexGuard<'a, T>> {
        self.wait_optional_timeout(guard, Some(timeout))
    }

    fn wait_optional_timeout<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        timeout: Option<Duration>,
    ) -> Option<MutexGuard<'a, T>> {
        let expected = self.futex.load(core::sync::atomic::Ordering::Relaxed);
        let mutex = guard.mutex();

        let suc = futex_wait(&self.futex, expected, timeout);

        let new_guard = mutex.lock();

        if !suc && timeout.is_some() {
            None
        } else {
            Some(new_guard)
        }
    }
}
