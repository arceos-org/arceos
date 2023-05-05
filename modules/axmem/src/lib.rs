#![cfg_attr(not(test), no_std)]
#![feature(drain_filter)]
extern crate alloc;

pub mod areas;
pub mod memory_set;
pub mod paging;
