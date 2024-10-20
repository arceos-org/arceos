use core::fmt;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering::Relaxed;

#[cfg(feature = "irq")]
use core::time::Duration;

use axtask::WaitQueue;

use crate::{mutex, MutexGuard};

/// A type indicating whether a timed wait on a condition variable returned
/// due to a time out or not.
///
/// It is returned by the [`wait_timeout`] method.
///
/// [`wait_timeout`]: Condvar::wait_timeout
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct WaitTimeoutResult(bool);

impl WaitTimeoutResult {
    /// Returns `true` if the wait was known to have timed out.
    #[must_use]
    pub fn timed_out(&self) -> bool {
        self.0
    }
}

/// A Condition Variable
///
/// Condition variables represent the ability to block a thread such that it
/// consumes no CPU time while waiting for an event to occur. Condition
/// variables are typically associated with a boolean predicate (a condition)
/// and a mutex. The predicate is always verified inside of the mutex before
/// determining that a thread must block.
///
/// Functions in this module will block the current **thread** of execution.
/// Note that any attempt to use multiple mutexes on the same condition
/// variable may result in a runtime panic.
pub struct Condvar {
    // The value of this atomic is simply incremented on every notification.
    // This is used by `.wait()` to not miss any notifications after
    // unlocking the mutex and before waiting for notifications.
    notify_counter: AtomicU32,
    wq: WaitQueue,
}

impl Condvar {
    /// Creates a new condition variable which is ready to be waited on and
    /// notified.
    #[must_use]
    #[inline]
    pub const fn new() -> Condvar {
        Self {
            notify_counter: AtomicU32::new(0),
            wq: WaitQueue::new(),
        }
    }

    /// Blocks the current thread until this condition variable receives a
    /// notification.
    ///
    /// This function will atomically unlock the mutex specified (represented by
    /// `guard`) and block the current thread. This means that any calls
    /// to [`notify_one`] or [`notify_all`] which happen logically after the
    /// mutex is unlocked are candidates to wake this thread up. When this
    /// function call returns, the lock specified will have been re-acquired.
    ///
    /// Note that this function is susceptible to spurious wakeups. Condition
    /// variables normally have a boolean predicate associated with them, and
    /// the predicate must always be checked each time this function returns to
    /// protect against spurious wakeups.
    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        // Examine the notification counter _before_ we unlock the mutex.
        let expected_counter = self.notify_counter.load(Relaxed);

        let mutex = mutex::guard_lock(&guard);
        // Unlock the mutex before going to sleep.
        unsafe {
            mutex.force_unlock();
        }
        // Wait, but only if there hasn't been any
        // notification since we unlocked the mutex.
        if self.notify_counter.load(Relaxed) == expected_counter {
            self.wq.wait();
        }
        // Lock the mutex again.
        mutex.lock();

        guard
    }

    /// Blocks the current thread until the provided condition becomes false.
    ///
    /// `condition` is checked immediately; if not met (returns `true`), this
    /// will [`wait`] for the next notification then check again. This repeats
    /// until `condition` returns `false`, in which case this function returns.
    ///
    /// This function will atomically unlock the mutex specified (represented by
    /// `guard`) and block the current thread. This means that any calls
    /// to [`notify_one`] or [`notify_all`] which happen logically after the
    /// mutex is unlocked are candidates to wake this thread up. When this
    /// function call returns, the lock specified will have been re-acquired.
    pub fn wait_while<'a, T, F>(
        &self,
        mut guard: MutexGuard<'a, T>,
        mut condition: F,
    ) -> MutexGuard<'a, T>
    where
        F: FnMut(&mut T) -> bool,
    {
        while condition(&mut *guard) {
            guard = self.wait(guard);
        }
        guard
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration.
    ///
    /// The semantics of this function are equivalent to [`wait`] except that
    /// the thread will be blocked for roughly no longer than `dur`. This
    /// method should not be used for precise timing due to anomalies such as
    /// preemption or platform differences that might not cause the maximum
    /// amount of time waited to be precisely `dur`.
    ///
    /// Note that the best effort is made to ensure that the time waited is
    /// measured with a monotonic clock, and not affected by the changes made to
    /// the system time. This function is susceptible to spurious wakeups.
    /// Condition variables normally have a boolean predicate associated with
    /// them, and the predicate must always be checked each time this function
    /// returns to protect against spurious wakeups. Additionally, it is
    /// typically desirable for the timeout to not exceed some duration in
    /// spite of spurious wakes, thus the sleep-duration is decremented by the
    /// amount slept. Alternatively, use the `wait_timeout_while` method
    /// to wait with a timeout while a predicate is true.
    ///
    /// The returned [`WaitTimeoutResult`] value indicates if the timeout is
    /// known to have elapsed.
    ///
    /// Like [`wait`], the lock specified will be re-acquired when this function
    /// returns, regardless of whether the timeout elapsed or not.
    #[cfg(feature = "irq")]
    pub fn wait_timeout<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        dur: Duration,
    ) -> (MutexGuard<'a, T>, WaitTimeoutResult) {
        // Examine the notification counter _before_ we unlock the mutex.
        let expected_counter = self.notify_counter.load(Relaxed);

        let mutex = mutex::guard_lock(&guard);
        // Unlock the mutex before going to sleep.
        unsafe {
            mutex.force_unlock();
        }

        let success = if self.notify_counter.load(Relaxed) == expected_counter {
            !self.wq.wait_timeout(dur)
        } else {
            true
        };

        // Lock the mutex again.
        mutex.lock();

        (guard, WaitTimeoutResult(!success))
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration.
    ///
    /// The semantics of this function are equivalent to [`wait_while`] except
    /// that the thread will be blocked for roughly no longer than `dur`. This
    /// method should not be used for precise timing due to anomalies such as
    /// preemption or platform differences that might not cause the maximum
    /// amount of time waited to be precisely `dur`.
    ///
    /// Note that the best effort is made to ensure that the time waited is
    /// measured with a monotonic clock, and not affected by the changes made to
    /// the system time.
    ///
    /// The returned [`WaitTimeoutResult`] value indicates if the timeout is
    /// known to have elapsed without the condition being met.
    ///
    /// Like [`wait_while`], the lock specified will be re-acquired when this
    /// function returns, regardless of whether the timeout elapsed or not.
    ///
    /// [`wait_while`]: Self::wait_while
    /// [`wait_timeout`]: Self::wait_timeout
    #[cfg(feature = "irq")]
    pub fn wait_timeout_while<'a, T, F>(
        &self,
        mut guard: MutexGuard<'a, T>,
        dur: Duration,
        mut condition: F,
    ) -> (MutexGuard<'a, T>, WaitTimeoutResult)
    where
        F: FnMut(&mut T) -> bool,
    {
        use axhal::time::monotonic_time;
        let start = monotonic_time();
        loop {
            if !condition(&mut *guard) {
                return (guard, WaitTimeoutResult(false));
            }
            let timeout = match dur.checked_sub(monotonic_time() - start) {
                Some(timeout) => timeout,
                None => return (guard, WaitTimeoutResult(true)),
            };
            guard = self.wait_timeout(guard, timeout).0;
        }
    }

    /// Wakes up one blocked thread on this condvar.
    ///
    /// If there is a blocked thread on this condition variable, then it will
    /// be woken up from its call to [`wait`] or [`wait_timeout`]. Calls to
    /// `notify_one` are not buffered in any way.
    ///
    /// To wake up all threads, see [`notify_all`].
    ///
    /// [`wait`]: Self::wait
    /// [`wait_timeout`]: Self::wait_timeout
    /// [`notify_all`]: Self::notify_all
    pub fn notify_one(&self) {
        self.notify_counter.fetch_add(1, Relaxed);
        self.wq.notify_one(true);
    }

    /// Wakes up all blocked threads on this condvar.
    ///
    /// This method will ensure that any current waiters on the condition
    /// variable are awoken. Calls to `notify_all()` are not buffered in any
    /// way.
    ///
    /// To wake up only one thread, see [`notify_one`].
    ///
    /// [`notify_one`]: Self::notify_one
    pub fn notify_all(&self) {
        self.notify_counter.fetch_add(1, Relaxed);
        self.wq.notify_all(true);
    }
}

impl fmt::Debug for Condvar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Condvar").finish_non_exhaustive()
    }
}

impl Default for Condvar {
    /// Creates a `Condvar` which is ready to be waited on and notified.
    fn default() -> Condvar {
        Condvar::new()
    }
}
