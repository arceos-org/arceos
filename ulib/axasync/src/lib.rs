//! # The ArceOS Asynchronous Library
//!
//! The [ArceOS] Asynchronous Library is a library for ArceOS to provide async
//! functionality other than standard library. Currently is mainly used for embassy runtime.
//!
//! ## Cargo Features
//!
//! - Utils
//!     - `time`: Enable `Embassy` time related support.
//!     - `sync`: Enable `Embassy` sync related support.
//! - Executor
//!     - `single`: Use the single-thread executor.
//!     - `thread`: Use the multi-thread executor.
//!     - `preempt`: Use the preemptive executor.
//!
//! [ArceOS]: https://github.com/arceos-org/arceos
//!
#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]

/// The embassy executor.
#[cfg(any(feature = "thread", feature = "preempt", feature = "single"))]
pub mod executor {
    use arceos_api::embassy_async as api;

    pub use api::AxExecutor as Executor;
    pub use embassy_executor::*;
    pub use embassy_futures::*;

    #[cfg(feature = "preempt")]
    pub use api::AxPrioFuture as PrioFuture;

    #[cfg(feature = "thread")]
    pub use api::{ax_block_on as block_on, ax_spawner as spawner};
}

/// The embassy time related functionality.
#[cfg(feature = "time")]
pub mod time {
    pub use embassy_time::*;
}

/// The embassy sync related functionality.
#[cfg(feature = "sync")]
pub mod sync {
    pub use embassy_sync::*;
}

/// The static cell.
pub mod cell {
    pub use static_cell::{ConstStaticCell, StaticCell};
}

// #[cfg(all(any(feature = "thread", feature = "preempt"), feature = "single"))]
// compile_error!(r#"feature "executor-thread" and "executor-single" are mutually exclusive"#);
