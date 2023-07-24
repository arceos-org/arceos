//! Exported C bindings, to call ArceOS funtions from C code.

#![allow(clippy::missing_safety_doc)]

/// cbindgen:ignore
#[rustfmt::skip]
#[path = "./ctypes_gen.rs"]
#[allow(dead_code, non_camel_case_types, non_upper_case_globals, clippy::upper_case_acronyms)]
mod ctypes;

#[macro_use]
mod utils;

#[cfg(feature = "fd")]
mod fd_ops;
#[cfg(feature = "fs")]
mod file;
#[cfg(any(feature = "select", feature = "epoll"))]
mod io_mpx;
#[cfg(feature = "alloc")]
mod malloc;
#[cfg(feature = "pipe")]
mod pipe;
#[cfg(feature = "multitask")]
mod pthread;
#[cfg(feature = "net")]
mod socket;
#[cfg(feature = "alloc")]
mod strftime;
#[cfg(feature = "fp_simd")]
mod strtod;
#[cfg(feature = "fd")]
mod uio;

mod errno;
mod mktime;
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
pub use self::strftime::ax_strftime;

#[cfg(feature = "fd")]
pub use self::fd_ops::{ax_close, ax_dup, ax_dup3, ax_fcntl, ax_fstat, ax_read, ax_write};
#[cfg(feature = "fd")]
pub use self::uio::ax_writev;

#[cfg(feature = "fs")]
pub use self::file::{ax_getcwd, ax_lseek, ax_lstat, ax_open, ax_stat};

#[cfg(feature = "net")]
pub use self::socket::{
    ax_accept, ax_bind, ax_connect, ax_getaddrinfo, ax_getpeername, ax_getsockname, ax_listen,
    ax_recv, ax_recvfrom, ax_send, ax_sendto, ax_shutdown, ax_socket,
};

#[cfg(feature = "multitask")]
pub use self::pthread::mutex::{
    ax_pthread_mutex_init, ax_pthread_mutex_lock, ax_pthread_mutex_unlock,
};
#[cfg(feature = "multitask")]
pub use self::pthread::{ax_getpid, ax_pthread_create, ax_pthread_exit, ax_pthread_join};

#[cfg(feature = "pipe")]
pub use self::pipe::ax_pipe;

#[cfg(feature = "select")]
pub use self::io_mpx::ax_select;

#[cfg(feature = "epoll")]
pub use self::io_mpx::{ax_epoll_create, ax_epoll_ctl, ax_epoll_wait};

#[cfg(feature = "fp_simd")]
pub use self::strtod::{ax_strtod, ax_strtof};

pub use self::errno::ax_errno_string;
pub use self::mktime::ax_mktime;
pub use self::stdio::{ax_print_str, ax_println_str};
pub use self::sys::ax_sysconf;
pub use self::time::{ax_clock_gettime, ax_nanosleep};
