#![no_std]
#![allow(clippy::new_ret_no_self)]

extern crate alloc;

mod disk;
pub mod fs;
mod highlevel;

pub use highlevel::*;
