#![no_std]
#![feature(const_trait_impl)]

mod linked_list;

pub mod unsafe_list;

pub use self::linked_list::{AdapterWrapped, List, Wrapper};
pub use unsafe_list::{Adapter, Cursor, Links};
