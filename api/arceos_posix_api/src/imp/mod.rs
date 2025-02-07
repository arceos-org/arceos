mod stdio;

pub mod io;
pub mod resources;
pub mod sys;
pub mod task;
pub mod time;

#[cfg(feature = "fd")]
pub mod fd_ops;
#[cfg(feature = "fs")]
pub mod fs;
#[cfg(any(feature = "select", feature = "epoll"))]
pub mod io_mpx;
#[cfg(feature = "net")]
pub mod net;
#[cfg(feature = "pipe")]
pub mod pipe;
#[cfg(feature = "multitask")]
pub mod pthread;

#[ctor_bare::register_ctor]
#[cfg(feature = "fd")]
fn init_stdio() {
    use crate::imp::fd_ops::FD_TABLE;
    use alloc::sync::Arc;
    use stdio::{stdin, stdout};
    let mut fd_table = flatten_objects::FlattenObjects::new();
    fd_table
        .add_at(0, Arc::new(stdin()) as _)
        .unwrap_or_else(|_| panic!()); // stdin
    fd_table
        .add_at(1, Arc::new(stdout()) as _)
        .unwrap_or_else(|_| panic!()); // stdout
    fd_table
        .add_at(2, Arc::new(stdout()) as _)
        .unwrap_or_else(|_| panic!()); // stderr
    FD_TABLE.init_new(spin::RwLock::new(fd_table));
}
