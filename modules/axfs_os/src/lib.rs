#![cfg_attr(not(test), no_std)]

pub mod file;
pub mod file_io;
pub mod stdio;
extern crate alloc;
pub mod flags;
pub use file::new_fd;
pub use stdio::{Stderr, Stdin, Stdout};
