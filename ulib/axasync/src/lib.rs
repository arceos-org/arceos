#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]

#[cfg(any(feature = "thread", feature = "preempt", feature = "single"))]
pub mod executor {
    use arceos_api::embassy_async as api;

    pub use embassy_executor::{main, task};

    pub use api::AxExecutor as Executor;
    pub use embassy_executor::*;
    pub use embassy_futures::*;

    #[cfg(feature = "preempt")]
    pub use api::AxPrioFuture as PrioFuture;

    #[cfg(feature = "thread")]
    pub use api::{ax_block_on as block_on, ax_spawner as spawner};
}

#[cfg(feature = "time")]
pub mod time {
    pub use embassy_time::*;
}

#[cfg(feature = "sync")]
pub mod sync {
    pub use embassy_sync::*;
}

pub mod cell {
    pub use static_cell::{ConstStaticCell, StaticCell};
}

#[cfg(all(any(feature = "thread", feature = "preempt"), feature = "single"))]
compile_error!(r#"feature "executor-thread" and "executor-single" are mutually exclusive"#);
