#![cfg_attr(not(test), no_std)]

pub use axlog::{debug, error, info, print, println, trace, warn};

#[cfg(feature = "alloc")]
extern crate alloc;
extern crate axlog;

#[cfg(not(test))]
extern crate axruntime;

pub mod io;
pub mod rand;
pub mod sync;
pub mod time;

#[cfg(feature = "multitask")]
pub mod task;

#[cfg(feature = "net")]
pub mod net;

#[cfg(feature = "display")]
pub mod display;
