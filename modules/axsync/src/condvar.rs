extern crate alloc;

use crate::MutexGuard;
use axtask::{Futex, futex_wait, futex_wake, futex_wake_all};

use core::{sync::atomic::AtomicU32, time::Duration};

/// A condition variable used for synchronizing threads based on a shared condition.
pub struct Condvar {
    futex: Futex,
}

impl Condvar {
    /// Creates a new [`Condvar`].
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            futex: AtomicU32::new(0),
        }
    }

    /// Notifies one waiting thread.
    ///
    /// If there are multiple threads waiting on this condition variable,
    /// only one of them will be woken up. The specific thread chosen is
    /// up to the scheduler's policy for the underlying `WaitQueue`.
    pub fn notify_one(&self) {
        self.futex
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        futex_wake(&self.futex);
    }

    /// Notifies all waiting threads.
    ///
    /// All threads currently waiting on this condition variable will be woken up.
    pub fn notify_all(&self) {
        self.futex
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        futex_wake_all(&self.futex);
    }

    /// Atomically unlocks the provided mutex guard and waits for a notification
    /// on this condition variable.
    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        // wait with no timeout should always return the guard
        self.wait_optional_timeout(guard, None)
            .expect("Condvar::wait with no timeout should not return None on timeout")
    }

    /// Atomically unlocks the provided mutex guard and waits for a notification
    /// on this condition variable, with a specified timeout.
    pub fn wait_timeout<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        timeout: Duration,
    ) -> Option<MutexGuard<'a, T>> {
        self.wait_optional_timeout(guard, Some(timeout))
    }

    #[inline]
    fn wait_optional_timeout<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        timeout: Option<Duration>,
    ) -> Option<MutexGuard<'a, T>> {
        let expected = self.futex.load(core::sync::atomic::Ordering::Relaxed);
        let mutex = lock_api::MutexGuard::mutex(&guard);

        let suc = futex_wait(&self.futex, expected, timeout);

        let new_guard = mutex.lock();

        if !suc && timeout.is_some() {
            None
        } else {
            Some(new_guard)
        }
    }
}
