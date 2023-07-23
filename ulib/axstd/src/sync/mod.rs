//! Useful synchronization primitives.

#[doc(no_inline)]
pub use core::sync::atomic;

#[cfg(feature = "alloc")]
#[doc(no_inline)]
pub use alloc::sync::{Arc, Weak};

#[cfg(feature = "multitask")]
pub use axsync::{Mutex, MutexGuard};

#[cfg(not(feature = "multitask"))]
pub use spinlock::{SpinNoIrq as Mutex, SpinNoIrqGuard as MutexGuard};
