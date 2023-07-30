#![no_std]

#[cfg(reexport_core)]
pub use core::sync::atomic::*;

#[cfg(not(reexport_core))]
mod polyfill;
#[cfg(not(reexport_core))]
pub use polyfill::*;
