//! [ArceOS](https://github.com/arceos-org/arceos) synchronization primitives.
//!
//! Currently supported primitives:
//!
//! - [`Mutex`]: A mutual exclusion primitive.
//! - mod [`spin`]: spinlocks imported from the [`kspin`] crate.
//! - [`Barrier`]: A barrier enables multiple threads to wait for each other.
//! - [`Condvar`]: A condition variable enables threads to wait until a particular.
//! - [`RwLock`]: A reader-writer lock.
//! - [`Semaphore`]: A semaphore is a synchronization primitive that controls access to a shared resource.
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
        mod barrier;
        mod condvar;
        mod mutex;
        mod semaphore;

        pub use self::barrier::{Barrier, BarrierWaitResult};
        pub use self::condvar::Condvar;
        #[doc(cfg(feature = "multitask"))]
        pub use self::mutex::{Mutex, MutexGuard};
        pub use semaphore::Semaphore;
    } else {
        #[doc(cfg(not(feature = "multitask")))]
        pub use kspin::{SpinNoIrq as Mutex, SpinNoIrqGuard as MutexGuard};
    }
}

mod rwlock;
pub use self::rwlock::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, Once};

    /// Used for initializing the only global scheduler for test environment.
    pub static INIT: Once = Once::new();
    /// Used for serializing the tests in this crate.
    pub static SEQ: Mutex<()> = Mutex::new(());
}
