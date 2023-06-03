//! Traits, helpers, and type definitions for core I/O functionality.

// TODO: stdio
//mod stdio;

pub use axio::prelude;
pub use axio::{BufRead, BufReader, Error, Read, Result, Seek, SeekFrom, Write};

#[macro_use]
pub mod logging;
use axerrno::{from_ret_code, AxError, AxResult};
use log::info;
use scheme::Stat;
use syscall_number::io::{OpenFlags, SEEK_CUR, SEEK_END, SEEK_SET};

use crate::syscall::io::{
    close, dup, fstat, fsync, lseek, open, read, remove_dir as remove_dir_inner,
    remove_file as remove_file_inner, write,
};
use crate::Mutex;
extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use self::env::absolute_path;

pub struct File {
    fd: usize,
}
impl File {
    pub fn open(path: &str) -> AxResult<Self> {
        from_ret_code(open(path, OpenFlags::READ)).map(|fd| Self { fd })
    }
    pub fn create(path: &str) -> AxResult<Self> {
        from_ret_code(open(
            path,
            OpenFlags::CREATE | OpenFlags::WRITE | OpenFlags::TRUNCATE,
        ))
        .map(|fd| Self { fd })
    }
    pub fn open_with(path: &str, flags: OpenFlags) -> AxResult<Self> {
        from_ret_code(open(path, flags)).map(|fd| Self { fd })
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
        from_ret_code(fsync(self.fd))?;
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
    pub fn stat(&mut self) -> AxResult<Stat> {
        let mut ret: Stat = Stat::new_file(0, 0);
        from_ret_code(fstat(self.fd, &mut ret))?;
        Ok(ret)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        close(self.fd);
    }
}

pub fn stdin() -> File {
    File::open("stdin:/").unwrap()
}
pub fn stdout() -> File {
    File::open("stdout:/").unwrap()
}

pub fn remove_dir(path: &str) -> AxResult<()> {
    from_ret_code(remove_dir_inner(path)).map(|_| ())
}

pub fn remove_file(path: &str) -> AxResult<()> {
    from_ret_code(remove_file_inner(path)).map(|_| ())
}

pub fn create_dir(path: &str) -> AxResult<()> {
    File::open_with(
        path,
        OpenFlags::CREATE | OpenFlags::EXCL | OpenFlags::DIRECTORY,
    )?;
    Ok(())
}

pub fn read_dir(path: &str) -> AxResult<Vec<String>> {
    let mut file = File::open_with(path, OpenFlags::READ | OpenFlags::DIRECTORY)?;
    let mut result = String::new();
    file.read_to_string(&mut result)?;
    Ok(result.split('\n').map(|x| x.to_string()).collect())
}

pub fn metadata(path: &str) -> AxResult<Stat> {
    let mut file = File::open(path)?;
    let ret = file.stat()?;
    info!("sdfsdf");
    Ok(ret)
}

static CURRENT_DIR_PATH: Mutex<String> = Mutex::new(String::new());
pub mod env {
    use axerrno::{ax_err, AxError, AxResult};
    extern crate alloc;
    use alloc::string::{String, ToString};

    use super::{scheme_helper, CURRENT_DIR_PATH};

    pub fn canonicalize(path: &str) -> AxResult<String> {
        Ok(axfs_vfs::path::canonicalize(path))
    }

    /// Returns the current working directory as a [`String`].
    pub fn current_dir() -> AxResult<String> {
        Ok(CURRENT_DIR_PATH.lock().clone())
    }

    /// Changes the current working directory to the specified path.
    pub fn set_current_dir(path: &str) -> AxResult<()> {
        let (scheme, path) = scheme_helper::get_scheme(path).ok_or(AxError::InvalidInput)?;
        if scheme != "file" {
            ax_err!(InvalidInput)?;
        }
        *CURRENT_DIR_PATH.lock() = absolute_path(path)?;
        Ok(())
    }

    pub fn absolute_path(path: &str) -> AxResult<String> {
        let res = if path.starts_with("/") {
            canonicalize(path)
        } else {
            let mut old = CURRENT_DIR_PATH.lock().clone();
            old.push_str("/");
            old.push_str(path);
            canonicalize(path)
        }?;
        if res.is_empty() {
            Ok("/".to_string())
        } else {
            Ok(res)
        }
    }
}

pub fn init() {
    *CURRENT_DIR_PATH.lock() = "/".to_string();
}

mod scheme_helper {
    pub fn get_scheme(url: &str) -> Option<(&str, &str)> {
        let mut url = url.splitn(2, ":");
        match (url.next(), url.next()) {
            (Some(scheme), Some(path)) => Some((scheme, path)),
            (Some(path), None) => Some(("file", path)),
            _ => None,
        }
    }
}

pub fn open_wrapper(path: &str, flags: OpenFlags) -> AxResult<usize> {
    let (scheme, path) = scheme_helper::get_scheme(path).ok_or(AxError::InvalidInput)?;
    if scheme == "file" {
        let absolute = absolute_path(path)?;
        from_ret_code(open(&absolute, flags))
    } else {
        from_ret_code(open(path, flags))
    }
}
