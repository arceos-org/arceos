//! Exported C bindings, to call ArceOS funtions from C code.

#![allow(clippy::missing_safety_doc)]

/// cbindgen:ignore
#[rustfmt::skip]
#[path = "./ctypes_gen.rs"]
#[allow(dead_code, non_camel_case_types, non_upper_case_globals, clippy::upper_case_acronyms)]
mod ctypes;

#[macro_use]
mod utils;

#[cfg(feature = "alloc")]
mod fd_ops;
#[cfg(feature = "fs")]
mod file;
#[cfg(feature = "alloc")]
mod malloc;
#[cfg(feature = "pipe")]
mod pipe;
#[cfg(feature = "net")]
mod socket;
#[cfg(feature = "multitask")]
mod thread;

mod setjmp;
mod stdio;
mod sys;
mod time;

/// Sets the seed for the random number generator.
#[no_mangle]
pub unsafe extern "C" fn ax_srand(seed: u32) {
    crate::rand::srand(seed);
}

/// Returns a 32-bit unsigned pseudo random interger.
#[no_mangle]
pub unsafe extern "C" fn ax_rand_u32() -> u32 {
    crate::rand::rand_u32()
}

/// Abort the current process.
#[no_mangle]
pub unsafe extern "C" fn ax_panic() -> ! {
    panic!()
}

/// Exits the current thread.
#[no_mangle]
pub unsafe extern "C" fn ax_exit(exit_code: core::ffi::c_int) -> ! {
    crate::thread::exit(exit_code)
}

#[cfg(feature = "alloc")]
pub use self::malloc::{ax_free, ax_malloc};

#[cfg(feature = "alloc")]
pub use self::fd_ops::{ax_close, ax_dup, ax_dup3, ax_fcntl, ax_fstat, ax_read, ax_write};

#[cfg(feature = "fs")]
pub use self::file::{ax_getcwd, ax_lseek, ax_lstat, ax_open, ax_stat};

#[cfg(feature = "net")]
pub use self::socket::{
    ax_accept, ax_bind, ax_connect, ax_listen, ax_recv, ax_recvfrom, ax_resolve_sockaddr, ax_send,
    ax_sendto, ax_shutdown, ax_socket,
};

#[cfg(feature = "multitask")]
pub use self::thread::ax_getpid;

#[cfg(feature = "pipe")]
pub use self::pipe::ax_pipe;

pub use self::stdio::{ax_print_str, ax_println_str};
pub use self::sys::ax_sysconf;
pub use self::time::{ax_clock_gettime, ax_nanosleep};
