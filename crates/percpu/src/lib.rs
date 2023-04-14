#![cfg_attr(target_os = "none", no_std)]
#![feature(doc_cfg)]

extern crate percpu_macros;

#[cfg_attr(feature = "sp-naive", path = "naive.rs")]
mod imp;

pub use self::imp::*;
pub use percpu_macros::def_percpu;

#[doc(hidden)]
pub mod __priv {
    #[cfg(feature = "preempt")]
    pub use kernel_guard::NoPreempt as NoPreemptGuard;
}
