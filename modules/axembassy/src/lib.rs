//! [ArceOS](https://github.com/arceos-org/arceos) embassy integration
//!
//! This module provides embassy asynchronous runtime integration, including
//! time driver, and executor with single-thread, multi-thread, preemptive(partially)
//! which are configurable by cargo features.
//!
//! # Cargo Features
//!
//! - `driver`: Enable time driver support. If it's enabled, time driver is used.
//!   Usually used by `axruntime` module in `irq` initiation.
//! - `executor-single`: Use the [single-thread executor][1]. It also enables the
//!   related utils modules.
//! - `executor-thread`: Use the [multi-thread executor][2]. It also enables the
//!   related utils modules and enables the `multitask` feature if it is enabled.
//! - `executor-preempt`: Use the [preemptive executor][3]. It also enables the
//!   related utils modules and enables the `executor-thread` feature if it is
//!   enabled.
//!
//! [1]: crate::executor::Executor
//! [2]: crate::asynch::spawner
//! [3]: crate::preempt::PrioFuture

#![cfg_attr(not(test), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]

cfg_if::cfg_if! {
    if #[cfg(any(feature = "executor-thread", feature = "executor-single"))] {
        extern crate alloc;
        extern crate log;

        mod delegate;
        #[cfg(feature= "executor-thread")]
        mod asynch;
        #[cfg(feature= "executor-preempt")]
        mod preempt;

        mod executor;
        mod executor_exports {
            pub use crate::executor::Executor;
            pub use crate::delegate::{Delegate, SameExecutorCell};
            pub use embassy_executor::Spawner;

            #[cfg(feature = "executor-thread")]
            pub use crate::asynch::{spawner,block_on};
            #[cfg(feature = "executor-thread")]
            pub use embassy_executor::SendSpawner;

            #[cfg(feature = "executor-preempt")]
            pub use crate::preempt::PrioFuture;
        }

        pub use executor_exports::*;
    }
}

#[cfg(feature = "driver")]
mod time_driver;

#[cfg(feature = "driver")]
pub use crate::time_driver::AxDriverAPI;

// #[cfg(all(
//     any(feature = "executor-thread", feature = "executor-preempt"),
//     feature = "executor-single"
// ))]
// compile_error!(
//     "feature `executor-thread`/`executor-preempt` and `executor-single` are mutually exclusive"
// );
