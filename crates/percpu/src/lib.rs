#![cfg_attr(target_os = "none", no_std)]

extern crate percpu_macros;

#[cfg_attr(feature = "sp-naive", path = "naive.rs")]
mod imp;

pub use self::imp::*;
pub use percpu_macros::def_percpu;
