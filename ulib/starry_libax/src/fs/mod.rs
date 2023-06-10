//! 实现文件描述符等相关文件操作

/// #![cfg_attr(not(test), no_std)]
pub mod file;

extern crate alloc;

pub mod dir;
pub mod flags;
pub mod link;
pub mod mount;
pub mod pipe;
pub mod types;

pub use axfs::api;
pub use axfs::monolithic_fs::{FileIO, FileIOType};
pub use axprocess::stdin::{Stderr, Stdin, Stdout};
pub use dir::{new_dir, DirDesc};
pub use file::{new_fd, FileDesc, FileMetaData};
pub use pipe::Pipe;
pub use types::{DirEnt, DirEntType, FilePath};
