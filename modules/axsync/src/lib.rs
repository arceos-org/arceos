#![cfg_attr(not(test), no_std)]
#![feature(const_trait_impl)]

pub use spinlock as spin;

#[cfg(feature = "multitask")]
mod mutex;

#[cfg(feature = "multitask")]
pub use self::mutex::{Mutex, MutexGuard};

#[cfg(not(feature = "multitask"))]
pub use spinlock::{SpinNoIrq as Mutex, SpinNoIrqGuard as MutexGuard};
