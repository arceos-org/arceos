#![cfg_attr(not(test), no_std)]

pub use axlog::{debug, error, info, trace, warn};

#[cfg(feature = "alloc")]
extern crate alloc;
extern crate axlog;

#[cfg(not(test))]
extern crate axruntime;

#[cfg(feature = "alloc")]
pub use alloc::{boxed, format, string, vec};

pub mod env;
pub mod io;
pub mod rand;
pub mod sync;
pub mod task;
pub mod time;

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "net")]
pub mod net;

#[cfg(feature = "display")]
pub mod display;
