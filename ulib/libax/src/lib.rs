#![cfg_attr(not(test), no_std)]

pub use axlog::{debug, error, info, trace, warn};
#[cfg(feature = "alloc")]
extern crate alloc;
extern crate axlog;

#[cfg(not(test))]
extern crate axruntime;

#[cfg(feature = "alloc")]
pub use alloc::{boxed, format, string, vec};
pub mod env;
pub mod io;
pub mod rand;
pub mod sync;
pub mod syscall;
pub use syscall::*;
pub mod task;
pub mod time;

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "net")]
pub mod net;

#[cfg(feature = "display")]
pub mod display;

#[no_mangle]
fn __user_start() -> ! {
    // axlog::ax_println!("test!");
    // let output = "hello world!\n".as_bytes();
    // sys_write(1, output);
    // syscall(SYSCALL_EXIT, [0, 0, 0, 0, 0, 0]);
    extern "Rust" {
        fn main();
    }
    unsafe {
        main();
    }
    unreachable!()
}
