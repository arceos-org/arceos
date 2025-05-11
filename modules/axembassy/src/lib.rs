#![feature(impl_trait_in_assoc_type)]
#![cfg_attr(not(test), no_std)]
#![feature(doc_cfg)]

extern crate alloc;
#[cfg(feature = "executor")]
extern crate embassy_executor;
#[cfg(feature = "futures")]
extern crate embassy_futures;
extern crate log;

#[cfg(feature = "executor")]
mod executor;
#[cfg(feature = "executor")]
mod runtime;
#[cfg(feature = "driver")]
mod time_driver;
mod waker;

#[cfg(feature = "executor")]
pub use crate::executor::Executor;
#[cfg(feature = "executor")]
pub use crate::runtime::init;
#[cfg(feature = "driver")]
pub use crate::time_driver::{AxDriverAPI, embassy_update_timer};
#[cfg(feature = "executor")]
#[doc(no_inline)]
pub use embassy_executor::*;
#[cfg(feature = "futures")]
#[doc(no_inline)]
pub use embassy_futures::*;
