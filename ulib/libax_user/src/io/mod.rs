//! Traits, helpers, and type definitions for core I/O functionality.

// TODO: stdio
//mod stdio;

pub use axio::prelude;
pub use axio::{BufRead, BufReader, Error, Read, Result, Seek, SeekFrom, Write};

#[macro_use]
pub mod logging;
use axerrno::{from_ret_code, AxResult};
use syscall_number::io::OpenFlags;

use crate::syscall::io::{open, read, write, dup, close};

pub struct File {
    fd: usize,
}
impl File {
    pub fn open(path: &str) -> AxResult<Self> {
        from_ret_code(open(path, OpenFlags::empty())).map(|fd| Self { fd })
    }
    pub fn create(path: &str) -> AxResult<Self> {
        from_ret_code(open(path, OpenFlags::CREATE)).map(|fd| Self { fd })
    }
    pub fn read(&self, s: &mut [u8]) -> AxResult<usize> {
        from_ret_code(read(self.fd, s))
    }
    pub fn read_data<T>(&self, s: &mut T) -> AxResult<usize> {
        from_ret_code(read(self.fd, unsafe {
            core::slice::from_raw_parts_mut(s as *mut T as *mut u8, core::mem::size_of::<T>())
        }))
    }
    pub fn write(&self, s: &[u8]) -> AxResult<usize> {
        from_ret_code(write(self.fd, s))
    }
    pub fn write_data<T>(&self, s: &T) -> AxResult<usize> {
        from_ret_code(write(self.fd, unsafe {
            core::slice::from_raw_parts(s as *const T as *mut u8, core::mem::size_of::<T>())
        }))
    }
    pub fn dup(&self, buf: &str) -> AxResult<Self> {
        let fd = from_ret_code(dup(self.fd, buf))?;
        Ok(File { fd })
    }
}
impl Drop for File {
    fn drop(&mut self) {
        close(self.fd);
    }
}
