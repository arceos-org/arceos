//! [ArceOS](https://github.com/arceos-org/arceos) synchronization primitives.
//!
//! Currently supported primitives:
//!
//! - [`Mutex`]: A mutual exclusion primitive.
//! - mod [`spin`]: spinlocks imported from the [`kspin`] crate.
//!
//! # Cargo Features
//!
//! - `multitask`: For use in the multi-threaded environments. If the feature is
//!   not enabled, [`Mutex`] will be an alias of [`spin::SpinNoIrq`]. This
//!   feature is enabled by default.

#![cfg_attr(not(test), no_std)]
#![feature(doc_cfg)]

pub use kspin as spin;

cfg_if::cfg_if! {
    if #[cfg(feature = "multitask")] {
        mod mutex;
        #[doc(cfg(feature = "multitask"))]
        pub use self::mutex::{Mutex, MutexGuard};
    } else {
        #[doc(cfg(not(feature = "multitask")))]
        pub use kspin::{SpinNoIrq as Mutex, SpinNoIrqGuard as MutexGuard};
    }
}

mod barrier;
mod condvar;
mod rwlock;
mod semaphore;

pub use self::barrier::{Barrier, BarrierWaitResult};
pub use self::condvar::Condvar;
pub use self::rwlock::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
pub use semaphore::Semaphore;
