#![no_std]
#![allow(clippy::new_ret_no_self)]
#![feature(maybe_uninit_slice)]

extern crate alloc;

mod disk;
pub mod fs;
mod highlevel;

pub use highlevel::*;

// TODO(mizu): Unify `Mutex` usage in this module. Currently we have
// `spin::Mutex`, `axsync::Mutex` and `kspin::Spin*`. A hybrid spinlock mutex
// may be a good choice.
