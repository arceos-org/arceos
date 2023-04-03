//! `std::fs`-like high-level filesystem manipulation operations.

mod dir;
mod file;

pub use self::dir::{DirEntry, ReadDir};
pub use self::file::{File, FileType, Metadata, OpenOptions};

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

/// Given a path, query the file system to get information about a file,
/// directory, etc.
pub fn metadata(path: &str) -> io::Result<Metadata> {
    File::open(path)?.metadata()
}
