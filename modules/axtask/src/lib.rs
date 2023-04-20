//! [ArceOS](https://github.com/rcore-os/arceos) task management module.
//!
//! This module provides primitives for task management, including task
//! creation, scheduling, sleeping, termination, etc. The scheduler algorithm
//! is configurable by cargo features.
//!
//! # Cargo Features
//!
//! - `multitask`: Enable multi-task support. If this feature is disabled,
//!   complex task management will not be available, only a few simple APIs
//!   can be used such as [`yield_now()`].
//! - `preempt`: Enable preemptive scheduling.
//! - `sched_fifo`: Use the [FIFO cooperative scheduler][1]. It also enables the
//!   `multitask` feature if it is enabled. This feature is enabled by default.
//! - `sched_rr`: Use the [Round-robin preemptive scheduler][2]. It also enables
//!   the `multitask` and `preempt` features if it is enabled.
//!
//! [1]: scheduler::FifoScheduler
//! [2]: scheduler::RRScheduler

#![cfg_attr(not(test), no_std)]
#![feature(const_trait_impl)]
#![feature(doc_cfg)]

cfg_if::cfg_if! {
    if #[cfg(feature = "multitask")] {
        #[macro_use]
        extern crate log;
        extern crate alloc;
        mod run_queue;
        mod task;
        mod timers;
        mod wait_queue;
    }
}

#[cfg_attr(not(feature = "multitask"), path = "api_s.rs")]
mod api;

#[cfg(test)]
mod tests;

pub use self::api::yield_now;
#[doc(cfg(feature = "multitask"))]
pub use self::api::*;
