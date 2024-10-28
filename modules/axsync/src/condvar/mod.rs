//! The Condition Variable
//!
//! Implementation adapted from the 'RwLock' type of the standard library. See:
//! <https://doc.rust-lang.org/stable/std/sync/struct.Condvar.html>
//!
//! Note: [`Condvar`] is not available when the `multitask` feature is disabled.

#[cfg(not(feature = "multitask"))]
mod no_thread;
#[cfg(not(feature = "multitask"))]
pub use no_thread::Condvar;

#[cfg(feature = "multitask")]
mod multitask;
#[cfg(feature = "multitask")]
pub use multitask::Condvar;

/// A type indicating whether a timed wait on a condition variable returned
/// due to a time out or not.
///
/// It is returned by the [`wait_timeout`] method.
///
/// [`wait_timeout`]: Condvar::wait_timeout
#[cfg(feature = "irq")]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct WaitTimeoutResult(bool);

#[cfg(feature = "irq")]
impl WaitTimeoutResult {
    /// Returns `true` if the wait was known to have timed out.
    #[must_use]
    pub fn timed_out(&self) -> bool {
        self.0
    }
}

#[cfg(test)]
mod tests;
