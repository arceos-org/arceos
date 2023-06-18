//! I/O multiplexing:
//!
//! * [`select`](select::ax_select)
//! * [`epoll_create`](epoll::ax_epoll_create)
//! * [`epoll_ctl`](epoll::ax_epoll_ctl)
//! * [`epoll_wait`](epoll::ax_epoll_wait)

mod epoll;
mod select;

pub use self::epoll::{ax_epoll_create, ax_epoll_ctl, ax_epoll_wait};
pub use self::select::ax_select;
