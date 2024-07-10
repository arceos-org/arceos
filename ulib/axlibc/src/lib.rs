//! [ArceOS] user program library for C apps.
//!
//! ## Cargo Features
//!
//! - CPU
//!     - `smp`: Enable SMP (symmetric multiprocessing) support.
//!     - `fp_simd`: Enable floating point and SIMD support.
//! - Interrupts:
//!     - `irq`: Enable interrupt handling support.
//! - Memory
//!     - `alloc`: Enable dynamic memory allocation.
//!     - `tls`: Enable thread-local storage.
//! - Task management
//!     - `multitask`: Enable multi-threading support.
//! - Upperlayer stacks
//!     - `fs`: Enable file system support.
//!     - `net`: Enable networking support.
//! - Lib C functions
//!     - `fd`: Enable file descriptor table.
//!     - `pipe`: Enable pipe support.
//!     - `select`: Enable synchronous I/O multiplexing ([select]) support.
//!     - `epoll`: Enable event polling ([epoll]) support.
//!
//! [ArceOS]: https://github.com/arceos-org/arceos
//! [select]: https://man7.org/linux/man-pages/man2/select.2.html
//! [epoll]: https://man7.org/linux/man-pages/man7/epoll.7.html

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]
#![feature(naked_functions)]
#![feature(thread_local)]
#![allow(clippy::missing_safety_doc)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[path = "."]
mod ctypes {
    #[rustfmt::skip]
    #[path = "libctypes_gen.rs"]
    #[allow(dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals, clippy::upper_case_acronyms)]
    mod libctypes;

    pub use arceos_posix_api::ctypes::*;
    pub use libctypes::*;
}

#[macro_use]
mod utils;

#[cfg(feature = "fd")]
mod fd_ops;
#[cfg(feature = "fs")]
mod fs;
#[cfg(any(feature = "select", feature = "epoll"))]
mod io_mpx;
#[cfg(feature = "alloc")]
mod malloc;
#[cfg(feature = "net")]
mod net;
#[cfg(feature = "pipe")]
mod pipe;
#[cfg(feature = "multitask")]
mod pthread;
#[cfg(feature = "alloc")]
mod strftime;
#[cfg(feature = "fp_simd")]
mod strtod;

mod errno;
mod io;
mod mktime;
mod rand;
mod resource;
mod setjmp;
mod sys;
mod time;
mod unistd;

#[cfg(not(test))]
pub use self::io::write;
pub use self::io::{read, writev};

pub use self::errno::strerror;
pub use self::mktime::mktime;
pub use self::rand::{rand, random, srand};
pub use self::resource::{getrlimit, setrlimit};
pub use self::setjmp::{longjmp, setjmp};
pub use self::sys::sysconf;
pub use self::time::{clock_gettime, nanosleep};
pub use self::unistd::{abort, exit, getpid};

#[cfg(feature = "alloc")]
pub use self::malloc::{free, malloc};
#[cfg(feature = "alloc")]
pub use self::strftime::strftime;

#[cfg(feature = "fd")]
pub use self::fd_ops::{ax_fcntl, close, dup, dup2, dup3};

#[cfg(feature = "fs")]
pub use self::fs::{ax_open, fstat, getcwd, lseek, lstat, rename, stat};

#[cfg(feature = "net")]
pub use self::net::{
    accept, bind, connect, freeaddrinfo, getaddrinfo, getpeername, getsockname, listen, recv,
    recvfrom, send, sendto, shutdown, socket,
};

#[cfg(feature = "multitask")]
pub use self::pthread::{pthread_create, pthread_exit, pthread_join, pthread_self};
#[cfg(feature = "multitask")]
pub use self::pthread::{pthread_mutex_init, pthread_mutex_lock, pthread_mutex_unlock};

#[cfg(feature = "pipe")]
pub use self::pipe::pipe;

#[cfg(feature = "select")]
pub use self::io_mpx::select;
#[cfg(feature = "epoll")]
pub use self::io_mpx::{epoll_create, epoll_ctl, epoll_wait};

#[cfg(feature = "fp_simd")]
pub use self::strtod::{strtod, strtof};
