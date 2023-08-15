#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]
#![feature(naked_functions)]
#![feature(result_option_inspect)]

extern crate alloc;
#[cfg(not(test))]
extern crate axruntime;

#[allow(unused_imports)]
pub mod fs;
pub mod io;
pub mod syscall;

pub mod test;

mod trap;
