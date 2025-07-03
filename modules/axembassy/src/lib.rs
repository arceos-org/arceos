#![cfg_attr(not(test), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]

cfg_if::cfg_if! {
    if #[cfg(any(feature = "executor-thread", feature = "executor-single"))] {
        extern crate alloc;
        extern crate log;

        pub mod delegate;

        #[cfg(feature = "executor-thread")]
        mod executor_thread;
        #[cfg(feature = "executor-thread")]
        mod asynch;
        #[cfg(feature = "executor-thread")]
        mod executor_thread_exports {
            pub use crate::executor_thread::Executor;
            pub use crate::asynch::{spawner,block_on,Spawner,SendSpawner};
            #[cfg(feature = "executor-preempt")]
            pub use crate::preempt::PrioFuture;
        }
        #[cfg(feature = "executor-thread")]
        pub use executor_thread_exports::*;

        #[cfg(feature = "executor-single")]
        mod executor;
        #[cfg(feature = "executor-single")]
        mod executor_exports {
            pub use crate::executor::Executor;
            pub use crate::asynch::Spawner;
        }
        #[cfg(feature = "executor-single")]
        pub use executor_exports::*;
    }
}

mod preempt;

#[cfg(feature = "driver")]
mod time_driver;

#[cfg(feature = "driver")]
pub use crate::time_driver::AxDriverAPI;

#[cfg(all(
    any(feature = "executor-thread", feature = "executor-preempt"),
    feature = "executor-single"
))]
compile_error!(
    "feature `executor-thread`/`executor-preempt` and `executor-single` are mutually exclusive"
);
