//! `std::fs`-like high-level filesystem manipulation operations.

mod dir;
mod file;

pub use self::dir::{DirEntry, ReadDir};
pub use self::file::{File, FileType, Metadata, OpenOptions, Permissions};

use alloc::{string::String, vec::Vec};
use axio::{prelude::*, Result};

/// Returns an iterator over the entries within a directory.
pub fn read_dir(path: &str) -> Result<ReadDir> {
    ReadDir::new(path)
}

/// Returns the canonical, absolute form of a path with all intermediate
/// components normalized and symbolic links resolved.
pub fn canonicalize(path: &str) -> Result<String> {
    Ok(path.into()) // TODO
}

/// Returns the current working directory as a [`String`].
pub fn current_dir() -> Result<String> {
    crate::root::current_dir()
}

/// Changes the current working directory to the specified path.
pub fn set_current_dir(path: &str) -> Result {
    crate::root::set_current_dir(path)
}

/// Read the entire contents of a file into a bytes vector.
pub fn read(path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut bytes = Vec::with_capacity(size as usize);
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Read the entire contents of a file into a string.
pub fn read_to_string(path: &str) -> Result<String> {
    let mut file = File::open(path)?;
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut string = String::with_capacity(size as usize);
    file.read_to_string(&mut string)?;
    Ok(string)
}

/// Write a slice as the entire contents of a file.
pub fn write(path: &str, contents: &[u8]) -> Result {
    File::create(path)?.write_all(contents)
}

/// Given a path, query the file system to get information about a file,
/// directory, etc.
pub fn metadata(path: &str) -> Result<Metadata> {
    File::open(path)?.metadata()
}
