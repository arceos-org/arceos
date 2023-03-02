#![cfg_attr(not(test), no_std)]

pub use axlog::{debug, error, info, print, println, trace, warn};

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
extern crate axlog;

#[cfg(not(test))]
extern crate axruntime;

pub mod io;
pub mod rand;

#[cfg(feature = "multitask")]
pub mod task;

#[cfg(feature = "net")]
pub mod net;
