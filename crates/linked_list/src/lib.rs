//! Linked lists that supports arbitrary removal in constant time.
//!
//! It is based on the linked list implementation in [Rust-for-Linux][1].
//!
//! [1]: https://github.com/Rust-for-Linux/linux/blob/rust/rust/kernel/linked_list.rs

#![no_std]

mod linked_list;

pub mod unsafe_list;

pub use self::linked_list::{AdapterWrapped, List, Wrapper};
pub use unsafe_list::{Adapter, Cursor, Links};
