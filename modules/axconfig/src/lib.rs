//! Platform-specific constants and parameters for [ArceOS].
//!
//! Currently supported platforms can be found in the [platforms] directory of
//! the [ArceOS] root.
//!
//! [ArceOS]: https://github.com/rcore-os/arceos
//! [platforms]: https://github.com/rcore-os/arceos/tree/main/platforms

#![no_std]

#[rustfmt::skip]
mod config {
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}

pub use config::*;

/// End address of the whole physical memory.
pub const PHYS_MEMORY_END: usize = PHYS_MEMORY_BASE + PHYS_MEMORY_SIZE;
