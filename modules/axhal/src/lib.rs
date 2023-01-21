#![no_std]
#![feature(asm_const)]
#![feature(naked_functions)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

mod common;
mod platform;

pub use platform::*;
