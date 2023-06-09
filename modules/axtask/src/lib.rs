//! [ArceOS](https://github.com/rcore-os/arceos) task management module.
//!
//! This module provides primitives for task management, including task
//! creation, scheduling, sleeping, termination, etc. The scheduler algorithm
//! is configurable by cargo features.
//!
//! # Cargo Features
//!
//! - `multitask`: Enable multi-task support. If it's enabled, complex task
//!   management and scheduling is used, as well as more task-related APIs.
//!   Otherwise, only a few APIs with naive implementation is available.
//! - `irq`: Interrupts are enabled. If this feature is enabled, timer-based
//!    APIs can be used, such as [`sleep`], [`sleep_until`], and
//!    [`WaitQueue::wait_timeout`].
//! - `preempt`: Enable preemptive scheduling.
//! - `sched_fifo`: Use the [FIFO cooperative scheduler][1]. It also enables the
//!   `multitask` feature if it is enabled. This feature is enabled by default.
//! - `sched_rr`: Use the [Round-robin preemptive scheduler][2]. It also enables
//!   the `multitask` and `preempt` features if it is enabled.
//! - `sched_cfs`: Use the [Completely Fair Scheduler][3]. It also enables the
//!   the `multitask` and `preempt` features if it is enabled.
//!
//! [1]: scheduler::FifoScheduler
//! [2]: scheduler::RRScheduler
//! [3]: scheduler::CFScheduler

#![cfg_attr(not(test), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]

cfg_if::cfg_if! {
    if #[cfg(feature = "multitask")] {
        #[macro_use]
        extern crate log;
        extern crate alloc;
        mod wait_queue;
        #[cfg(feature = "irq")]
        mod timers;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "monolithic")] {
        pub mod monolithic_task;
    }
    else {
        mod task;
        mod run_queue;
    }
}

#[cfg_attr(not(feature = "multitask"), path = "api_s.rs")]
mod api;

#[cfg(test)]
mod tests;

#[doc(cfg(feature = "multitask"))]
pub use self::api::*;
pub use self::api::{sleep, sleep_until, yield_now};
