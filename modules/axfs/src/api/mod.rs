//! `std::fs`-like high-level filesystem manipulation operations.

mod dir;
mod file;

pub use self::dir::{DirEntry, ReadDir};
pub use self::file::{File, FileType, Metadata, OpenOptions, Permissions};

use alloc::string::String;
use axio as io;

/// Returns an iterator over the entries within a directory.
pub fn read_dir(path: &str) -> io::Result<ReadDir> {
    ReadDir::new(path)
}

/// Returns the canonical, absolute form of a path with all intermediate
/// components normalized and symbolic links resolved.
pub fn canonicalize(path: &str) -> io::Result<String> {
    Ok(path.into()) // TODO
}

/// Returns the current working directory as a [`String`].
pub fn current_dir() -> io::Result<String> {
    crate::root::current_dir()
}

/// Changes the current working directory to the specified path.
pub fn set_current_dir(path: &str) -> io::Result<()> {
    crate::root::set_current_dir(path)
}

/// Given a path, query the file system to get information about a file,
/// directory, etc.
pub fn metadata(path: &str) -> io::Result<Metadata> {
    File::open(path)?.metadata()
}
