#![no_std]
#![allow(missing_docs)]
/// Linked list
pub mod list;
/// Sync
pub mod sync;
/// macros
pub mod xmarco;

pub use list::{InListNode, ListNode};
pub use sync::{rw_spin_mutex, spin_mutex};
