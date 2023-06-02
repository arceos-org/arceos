#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]
#![feature(naked_functions)]
#![feature(result_option_inspect)]

#[cfg(not(test))]
extern crate axruntime;

#[allow(unused_imports)]
mod fs;
pub mod io;
pub mod syscall;

#[cfg(feature = "test")]
pub mod test;

mod trap;
