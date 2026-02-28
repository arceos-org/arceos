//! Useful synchronization primitives.

#[cfg(feature = "alloc")]
#[doc(no_inline)]
pub use alloc::sync::{Arc, Weak};
#[doc(no_inline)]
pub use core::sync::atomic;

#[cfg(feature = "multitask")]
mod mutex;

#[cfg(not(feature = "multitask"))]
#[doc(cfg(not(feature = "multitask")))]
pub use kspin::{SpinRaw as Mutex, SpinRawGuard as MutexGuard};

#[cfg(feature = "multitask")]
#[doc(cfg(feature = "multitask"))]
pub use self::mutex::{Mutex, MutexGuard}; // never used in IRQ context
