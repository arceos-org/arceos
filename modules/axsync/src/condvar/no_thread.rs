//! Dummy implementation of `Condvar` for single-threaded environments.
use crate::MutexGuard;

pub struct Condvar {}

impl Condvar {
    #[inline]
    pub const fn new() -> Condvar {
        Condvar {}
    }

    #[inline]
    pub fn notify_one(&self) {}

    #[inline]
    pub fn notify_all(&self) {}

    pub fn wait<'a, T>(&self, _guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        panic!("condvar wait not supported")
    }

    pub fn wait_while<'a, T, F>(
        &self,
        mut _guard: MutexGuard<'a, T>,
        mut _condition: F,
    ) -> MutexGuard<'a, T>
    where
        F: FnMut(&mut T) -> bool,
    {
        panic!("condvar wait_while not supported")
    }

    #[cfg(feature = "irq")]
    pub fn wait_timeout<'a, T>(
        &self,
        _guard: MutexGuard<'a, T>,
        _dur: core::time::Duration,
    ) -> (MutexGuard<'a, T>, WaitTimeoutResult) {
        panic!("condvar wait_timeout not supported")
    }

    #[cfg(feature = "irq")]
    pub fn wait_timeout_while<'a, T, F>(
        &self,
        mut _guard: MutexGuard<'a, T>,
        _dur: core::time::Duration,
        mut _condition: F,
    ) -> (MutexGuard<'a, T>, WaitTimeoutResult)
    where
        F: FnMut(&mut T) -> bool,
    {
        panic!("condvar wait_timeout_while not supported")
    }
}
