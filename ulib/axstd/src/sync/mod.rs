//! Useful synchronization primitives.

#[cfg(feature = "multitask")]
pub use axsync::{Mutex, MutexGuard};

#[cfg(feature = "multitask")]
pub use axtask::WaitQueue;

pub use spinlock as spin;

#[cfg(not(feature = "multitask"))]
pub use spinlock::{SpinNoIrq as Mutex, SpinNoIrqGuard as MutexGuard};
