//! `std::fs`-like filesystem manipulation operations.

mod dir;
mod file;

pub use self::dir::{DirEntry, ReadDir};
pub use self::file::{File, FileType, OpenOptions};

use alloc::string::String;

/// Returns an iterator over the entries within a directory.
pub fn read_dir(path: &str) -> axio::Result<ReadDir> {
    ReadDir::new(path)
}

/// Returns the canonical, absolute form of a path with all intermediate
/// components normalized and symbolic links resolved.
pub fn canonicalize(path: &str) -> axio::Result<String> {
    Ok(path.into()) // TODO
}
