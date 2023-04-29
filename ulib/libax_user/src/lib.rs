#![no_std]

mod syscall;
pub use syscall::*;

#[macro_use]
pub mod logging;
pub use logging::__print_impl;
pub use logging::{debug, error, info, trace, warn};

mod allocate;
mod entry;
pub mod rand;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    task::exit(1);
}
