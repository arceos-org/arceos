#![feature(impl_trait_in_assoc_type)]
#![cfg_attr(not(test), no_std)]
#![feature(doc_cfg)]

extern crate alloc;
extern crate log;

#[cfg(feature = "executor")]
mod executor;
#[cfg(feature = "driver")]
mod time_driver;
// mod waker;
mod asynch;

#[cfg(feature = "executor")]
pub use crate::asynch::{block_on, spawner};
#[cfg(feature = "executor")]
pub use crate::executor::Executor;
use axtask::spawn_raw;
#[cfg(feature = "executor")]
#[doc(no_inline)]
pub use embassy_executor::*;
#[cfg(feature = "executor")]
#[doc(no_inline)]
pub use embassy_futures::*;

#[cfg(feature = "driver")]
pub use crate::time_driver::AxDriverAPI;

pub fn spawn_init() {
    spawn_raw(init, "async".into(), 4096);
}

fn init() {
    use static_cell::StaticCell;

    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    EXECUTOR
        .init_with(Executor::new)
        .run(|sp| sp.must_spawn(init_task()));
}

#[embassy_executor::task]
async fn init_task() {
    use log::debug;

    let spawner = asynch::Spawner::for_current_executor().await;
    asynch::set_spawner(spawner.make_send());

    debug!("axembassy::init_task()");
}
