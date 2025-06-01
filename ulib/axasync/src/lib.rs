#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]

use arceos_api::embassy_async as api;

#[cfg(any(feature = "thread", feature = "single"))]
pub use api::{AxExecutor as Executor, AxSpawner as Spawner};

#[cfg(feature = "thread")]
pub use api::{AxSendSpawner as SendSpawner, ax_block_on as block_on, ax_spawner as spawner};
