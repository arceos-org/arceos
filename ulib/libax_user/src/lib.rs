#![cfg_attr(all(not(test), not(doc), target_os = "none"), no_std)]

mod syscall;
pub use syscall::*;

pub use io::logging;
pub use io::logging::__print_impl;
pub use logging::{debug, error, info, trace, warn};

#[macro_use]
pub mod io;

mod allocate;
mod entry;
#[path = "../../libax/src/rand.rs"]
pub mod rand;
mod sync;
pub use sync::{Mutex, MutexGuard};

#[cfg(all(target_os = "none", not(test)))]
use core::panic::PanicInfo;

#[cfg(all(target_os = "none", not(test)))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    task::exit(1);
}

pub mod scheme {
    pub use scheme::*;
}
pub mod axerrno {
    pub use axerrno::*;
}
pub use syscall_number::io::OpenFlags;

//#[cfg(feature = "net")]
//pub mod net;
