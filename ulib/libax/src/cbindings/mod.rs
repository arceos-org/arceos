//! Exported C bindings, to call ArceOS funtions from C code.

#![allow(clippy::missing_safety_doc)]

#[macro_use]
mod utils;

#[cfg(feature = "alloc")]
mod fd_table;
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
mod sys;
mod time;

/// cbindgen:ignore
#[rustfmt::skip]
#[path = "./ctypes_gen.rs"]
#[allow(dead_code, non_camel_case_types, non_upper_case_globals, clippy::upper_case_acronyms)]
mod ctypes;

use crate::io::Write;
use core::ffi::{c_char, c_int};

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

use spinlock::SpinNoIrq;
// Lock used by ax_println_str for C apps
static LOCK: SpinNoIrq<()> = SpinNoIrq::new(());

/// Print a string to the global standard output stream.
#[no_mangle]
pub unsafe extern "C" fn ax_print_str(buf: *const c_char, count: usize) -> c_int {
    if buf.is_null() {
        return -axerrno::LinuxError::EFAULT.code();
    }

    let bytes = unsafe { core::slice::from_raw_parts(buf as *const u8, count as _) };
    crate::io::stdout()
        .write(bytes)
        .unwrap_or_else(|e| -axerrno::LinuxError::from(e).code() as _) as _
}

/// Printf a string to the global standard output stream. Add a line break.
#[no_mangle]
pub unsafe extern "C" fn ax_println_str(buf: *const c_char, count: usize) -> c_int {
    if buf.is_null() {
        return -axerrno::LinuxError::EFAULT.code();
    }

    let _guard = LOCK.lock();
    let bytes = unsafe { core::slice::from_raw_parts(buf as *const u8, count as _) };

    (|| -> axerrno::LinuxResult<c_int> {
        let r = crate::io::stdout().write(bytes)? as c_int;
        crate::io::stdout().write(b"\n")?;
        Ok(r + 1)
    })()
    .unwrap_or_else(|e| -e.code())
}

/// Abort the current process.
#[no_mangle]
pub unsafe extern "C" fn ax_panic() -> ! {
    panic!()
}

/// Exits the current thread.
#[no_mangle]
pub unsafe extern "C" fn ax_exit(exit_code: c_int) -> ! {
    crate::thread::exit(exit_code)
}

#[cfg(feature = "alloc")]
pub use self::malloc::{ax_free, ax_malloc};

#[cfg(feature = "alloc")]
pub use self::fd_table::{ax_close, ax_dup, ax_dup3, ax_fcntl, ax_fstat, ax_read, ax_write};

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

pub use self::sys::ax_sysconf;
pub use self::time::{ax_clock_gettime, ax_nanosleep};
