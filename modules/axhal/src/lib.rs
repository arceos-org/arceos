#![no_std]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(const_trait_impl)]
#![feature(const_maybe_uninit_zeroed)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

mod platform;

pub mod arch;
pub mod mem;
pub mod time;

#[cfg(feature = "paging")]
pub mod paging;

pub mod console {
    pub use super::platform::console::*;
}

pub mod misc {
    pub use super::platform::misc::*;
}
