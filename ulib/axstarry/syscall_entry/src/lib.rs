#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]
#![feature(naked_functions)]
#![feature(result_option_inspect)]
#![deny(warnings)]
#![feature(ip_in_core)]
/// 需要手动引入这个库，否则会报错：`#[panic_handler]` function required, but not found.
extern crate axruntime;

mod trap;

mod syscall;
mod test;

mod fs;

pub use test::run_testcases;
