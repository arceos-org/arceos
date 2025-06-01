#![cfg_attr(not(test), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]

cfg_if::cfg_if! {
    if #[cfg(any(feature = "executor-thread", feature = "executor-single"))] {
        extern crate alloc;
        extern crate log;

        pub mod delegate;
        pub mod asynch;

        #[cfg(feature = "executor-thread")]
        mod executor_thread;
        #[cfg(feature = "executor-single")]
        mod executor;

        #[cfg(feature = "executor-thread")]
        pub use crate::executor_thread::Executor;
        #[cfg(feature = "executor-single")]
        pub use crate::executor::Executor;

        pub use crate::asynch::{Spawner, SendSpawner};
        #[cfg(feature = "executor-thread")]
        pub use crate::asynch::{spawner,block_on};

        #[cfg(feature = "executor-thread")]
        pub fn init_spawn() {
            use axtask::spawn_raw;
            spawn_raw(init, "async".into(), axconfig::TASK_STACK_SIZE);
        }

        #[cfg(feature = "executor-thread")]
        pub fn init() {
            use crate::executor_thread::Executor;
            use static_cell::StaticCell;

            static EXECUTOR: StaticCell<Executor> = StaticCell::new();
            EXECUTOR
                .init_with(Executor::new)
                .run(|sp| sp.must_spawn(init_task()));
        }

        #[cfg(feature = "executor-thread")]
        #[embassy_executor::task]
        async fn init_task() {
            use axtask::unpark_task;
            use log::info;

            let spawner = asynch::Spawner::for_current_executor().await;
            asynch::set_spawner(spawner.make_send());
            info!("spawner is set, unpark the main thread.");
            unpark_task(2, true);
        }
    }
}

#[cfg(feature = "driver")]
mod time_driver;

#[cfg(feature = "driver")]
pub use crate::time_driver::AxDriverAPI;

#[cfg(all(feature = "executor-thread", feature = "executor-single"))]
compile_error!("feature `executor-thread` and `executor-single` are mutually exclusive");
