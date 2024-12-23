//! Platform-specific constants and parameters for [ArceOS].
//!
//! Currently supported platforms can be found in the [platforms] directory of
//! the [ArceOS] root.
//!
//! [ArceOS]: https://github.com/arceos-org/arceos
//! [platforms]: https://github.com/arceos-org/arceos/tree/main/platforms

#![no_std]

mod config {
    axconfig_gen_macros::include_configs!("../../.axconfig.toml");
}

pub use config::*;
