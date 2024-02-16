#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]
#![feature(naked_functions)]
#![deny(warnings)]

/// 需要手动引入这个库，否则会报错：`#[panic_handler]` function required, but not found.
extern crate axruntime;

mod trap;

mod syscall;
mod test;

#[cfg(feature = "ext4fs")]
#[allow(unused_imports)]
use axlibc::ax_open;

pub use test::run_testcases;
