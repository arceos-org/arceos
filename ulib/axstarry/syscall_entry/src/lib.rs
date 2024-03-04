#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]
#![feature(naked_functions)]
#![deny(warnings)]

/// 需要手动引入这个库，否则会报错：`#[panic_handler]` function required, but not found.
extern crate axruntime;

mod trap;

mod syscall;
extern crate alloc;
pub use axprocess::{
    link::{create_link, FilePath},
    wait_pid, Process,
};
pub use axprocess::{yield_now_task, PID2PC};
pub use syscall_utils::{new_file, FileFlags};
mod api;
pub use api::*;

#[cfg(feature = "ext4fs")]
#[allow(unused_imports)]
use axlibc::ax_open;
