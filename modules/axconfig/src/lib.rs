//! Platform-specific constants and parameters for [ArceOS].
//!
//! Currently supported platform configs can be found in the [configs] directory of
//! the [ArceOS] root.
//!
//! [ArceOS]: https://github.com/arceos-org/arceos
//! [configs]: https://github.com/arceos-org/arceos/tree/main/configs

#![no_std]

#[cfg(not(feature = "dyn"))]
axconfig_macros::include_configs!(path_env = "AX_CONFIG_PATH", fallback = "dummy.toml");

#[cfg(feature = "dyn")]
mod dyn_impl;

#[cfg(feature = "dyn")]
pub use dyn_impl::*;
