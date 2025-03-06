//! Useful synchronization primitives.

#[doc(no_inline)]
pub use core::sync::atomic;

#[cfg(feature = "alloc")]
#[doc(no_inline)]
pub use alloc::sync::{Arc, Weak};

// Re-export the `Mutex` and `MutexGuard` types.
pub use axsync::{Mutex, MutexGuard};

// Re-export the `RwLock` and related types.
pub use axsync::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

// Re-export the `Barrier` and `BarrierWaitResult` types.
#[cfg(feature = "multitask")]
pub use axsync::{Barrier, BarrierWaitResult};
// Re-export the `Condvar` and `WaitTimeoutResult` types.
#[cfg(feature = "multitask")]
pub use axsync::{Condvar, WaitTimeoutResult};
// Re-export the `Semaphore` and `SemaphoreGuard` types.
#[cfg(feature = "multitask")]
pub use axsync::{Semaphore, SemaphoreGuard};
