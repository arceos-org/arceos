#![cfg_attr(not(test), no_std)]

pub mod file;
pub mod file_io;
pub mod stdio;
extern crate alloc;
use alloc::format;
use alloc::vec::Vec;
pub mod flags;
use axerrno::{ax_err_type, AxResult};
use axfs::api::OpenOptions;
use axio::{Read, Seek, SeekFrom};
pub use file::new_fd;
pub use stdio::{Stderr, Stdin, Stdout};

/// Reads the content of file, Doesn't create a new file descriptor.
pub fn read_file(path: &str) -> AxResult<Vec<u8>> {
    let mut file = OpenOptions::new().read(true).open(path).map_err(|err| {
        ax_err_type!(
            Io,
            format!("failed to open file: {}, source: {}", path, err.as_str())
        )
    })?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|err| {
        ax_err_type!(
            Io,
            format!(
                "failed to read the file: {}, source: {}",
                path,
                err.as_str()
            )
        )
    })?;
    Ok(buf)
}

/// Reads a file from offest.
pub fn read_file_with_offset(path: &str, offset: isize) -> AxResult<Vec<u8>> {
    let mut file = OpenOptions::new().read(true).open(path).map_err(|err| {
        ax_err_type!(
            Io,
            format!("failed to open file: {}, source: {}", path, err.as_str())
        )
    })?;
    file.seek(SeekFrom::Start(offset as u64)).map_err(|err| {
        ax_err_type!(
            Io,
            format!(
                "failed to seek file: {} at {}, source: {}",
                path,
                offset,
                err.as_str()
            )
        )
    })?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|err| {
        ax_err_type!(
            Io,
            format!(
                "failed to read the file: {}, source: {}",
                path,
                err.as_str()
            )
        )
    })?;
    Ok(buf)
}
