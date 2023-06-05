//! I/O multiplexing:
//!
//! * [`select`](select::ax_select)

mod select;

pub use self::select::ax_select;
