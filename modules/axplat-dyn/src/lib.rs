#![no_std]
#![cfg(not(any(windows, unix)))]
#![feature(used_with_arg)]

extern crate somehal;

#[macro_use]
extern crate axplat;
#[allow(unused_imports)]
#[macro_use]
extern crate log;

mod boot;
mod console;
pub mod drivers;
mod generic_timer;
mod init;
#[cfg(feature = "irq")]
mod irq;
mod mem;
mod power;

#[cfg(not(feature = "irq"))]
#[somehal::irq_handler]
fn somehal_handle_irq(_irq: somehal::irq::IrqId) {}

// pub mod config {
//     //! Platform configuration module.
//     //!
//     //! If the `AX_CONFIG_PATH` environment variable is set, it will load the configuration from the specified path.
//     //! Otherwise, it will fall back to the `axconfig.toml` file in the current directory and generate the default configuration.
//     //!
//     //! If the `PACKAGE` field in the configuration does not match the package name, it will panic with an error message.
//     axconfig_macros::include_configs!(path_env = "AX_CONFIG_PATH", fallback = "axconfig.toml");
//     assert_str_eq!(
//         PACKAGE,
//         env!("CARGO_PKG_NAME"),
//         "`PACKAGE` field in the configuration does not match the Package name. Please check your configuration file."
//     );
// }
