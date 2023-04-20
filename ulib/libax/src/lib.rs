//! ArceOS user program library, with an interface similar to rust
//! [std](https://doc.rust-lang.org/std/), but calling the functions directly
//! in ArceOS modules, instead of using libc and system calls.

#![cfg_attr(not(test), no_std)]
#![feature(doc_auto_cfg)]

pub use axlog::{debug, error, info, trace, warn};

#[cfg(not(test))]
extern crate axruntime;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
#[doc(no_inline)]
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

#[cfg(feature = "cbindings")]
pub mod cbindings;
