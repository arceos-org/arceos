//! Traits, helpers, and type definitions for core I/O functionality.

// TODO: stdio
//mod stdio;

pub use axio::prelude;
pub use axio::{BufRead, BufReader, Error, Read, Result, Seek, SeekFrom, Write};

#[macro_use]
pub mod logging;
use axerrno::{from_ret_code, AxResult};
use syscall_number::io::{OpenFlags, SEEK_CUR, SEEK_END, SEEK_SET};

use crate::syscall::io::{close, dup, lseek, open, read, write};

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
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        from_ret_code(read(self.fd, buf))
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        from_ret_code(write(self.fd, buf))
    }
    fn flush(&mut self) -> AxResult {
        Ok(())
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> AxResult<u64> {
        let (offset, whence) = match pos {
            SeekFrom::Current(offset) => (offset as isize, SEEK_CUR),
            SeekFrom::End(offset) => (offset as isize, SEEK_END),
            SeekFrom::Start(offset) => (offset as isize, SEEK_SET),
        };
        from_ret_code(lseek(self.fd, offset, whence)).map(|x| x as u64)
    }
}

impl File {
    pub fn read_data<T>(&mut self, s: &mut T) -> AxResult<usize> {
        self.read(unsafe {
            core::slice::from_raw_parts_mut(s as *mut T as *mut u8, core::mem::size_of::<T>())
        })
    }
    pub fn write_data<T>(&mut self, s: &T) -> AxResult<usize> {
        self.write(unsafe {
            core::slice::from_raw_parts(s as *const T as *mut u8, core::mem::size_of::<T>())
        })
    }
    pub fn dup(&mut self, buf: &str) -> AxResult<Self> {
        let fd = from_ret_code(dup(self.fd, buf))?;
        Ok(File { fd })
    }
}

impl Drop for File {
    fn drop(&mut self) {
        close(self.fd);
    }
}
