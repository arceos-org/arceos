//! User library of the microkernel version of ArceOS.
//!
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

/// `Scheme` re-export
pub mod scheme {
    pub use scheme::*;
}
/// Error number re-export
pub mod axerrno {
    pub use axerrno::*;
}
pub use syscall_number::io::OpenFlags;

//#[cfg(feature = "net")]
//pub mod net;

pub use io::env;

// for macro
/// this macro wraps a loop-wait routine
/// where wait is indicated as `EAGAIN` / `EWOULDBLOCK`
#[macro_export]
macro_rules! loop_wait {
    ($e:expr) => {
        loop {
            match $e {
                Ok(value) => break Ok(value),
                Err($crate::axerrno::AxError::WouldBlock) => $crate::task::yield_now(),
                Err(e) => break Err(e),
            }
        }
    };
}
