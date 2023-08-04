//! Useful synchronization primitives.

#[doc(no_inline)]
pub use core::sync::atomic;

#[cfg(feature = "alloc")]
#[doc(no_inline)]
pub use alloc::sync::{Arc, Weak};

#[cfg(feature = "multitask")]
mod mutex;

#[cfg(feature = "multitask")]
#[doc(cfg(feature = "multitask"))]
pub use self::mutex::{Mutex, MutexGuard};

#[cfg(not(feature = "multitask"))]
#[doc(cfg(not(feature = "multitask")))]
pub use spinlock::{SpinRaw as Mutex, SpinRawGuard as MutexGuard}; // never used in IRQ context
