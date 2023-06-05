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

/// RAII style File objects
/// closes the file upon `drop`
pub struct File {
    fd: usize,
}
impl File {
    /// Opens a file (read only)
    pub fn open(path: &str) -> AxResult<Self> {
        open_wrapper(path, OpenFlags::READ).map(|fd| Self { fd })
    }
    /// Opens a file (write only, create one if not exist)
    pub fn create(path: &str) -> AxResult<Self> {
        open_wrapper(
            path,
            OpenFlags::CREATE | OpenFlags::WRITE | OpenFlags::TRUNCATE,
        )
        .map(|fd| Self { fd })
    }
    /// Opens a file with options specified in `flags`
    pub fn open_with(path: &str, flags: OpenFlags) -> AxResult<Self> {
        open_wrapper(path, flags).map(|fd| Self { fd })
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
    /// read data in type `T`
    pub fn read_data<T>(&mut self, s: &mut T) -> AxResult<usize> {
        self.read(unsafe {
            core::slice::from_raw_parts_mut(s as *mut T as *mut u8, core::mem::size_of::<T>())
        })
    }
    /// write data of type `T`
    pub fn write_data<T>(&mut self, s: &T) -> AxResult<usize> {
        self.write(unsafe {
            core::slice::from_raw_parts(s as *const T as *mut u8, core::mem::size_of::<T>())
        })
    }
    /// duplicate a file, see [scheme] for more information
    pub fn dup(&mut self, buf: &str) -> AxResult<Self> {
        let fd = from_ret_code(dup(self.fd, buf))?;
        Ok(File { fd })
    }
    /// get the metadata of the file
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

/// Return the `stdin` `File` wrapper
/// NOTE: fd 0 (stdin) will not be closed after drop
pub fn stdin() -> File {
    File::open("stdin:/").unwrap()
}
/// Return the `stdout` `File` wrapper
/// NOTE: fd 1 (stdout) will not be closed after drop
pub fn stdout() -> File {
    File::open("stdout:/").unwrap()
}

/// remove a directory, recursively delete is not supported
pub fn remove_dir(path: &str) -> AxResult<()> {
    op_wrapper(path, |path| remove_dir_inner(path)).map(|_| ())
}

/// remove (unlink) a file
pub fn remove_file(path: &str) -> AxResult<()> {
    op_wrapper(path, |path| remove_file_inner(path)).map(|_| ())
}

/// create a directory
pub fn create_dir(path: &str) -> AxResult<()> {
    File::open_with(
        path,
        OpenFlags::CREATE | OpenFlags::EXCL | OpenFlags::DIRECTORY,
    )?;
    Ok(())
}

/// show all items in a directory, return a `Vec` of names
pub fn read_dir(path: &str) -> AxResult<Vec<String>> {
    let mut file = File::open_with(path, OpenFlags::READ | OpenFlags::DIRECTORY)?;
    let mut result = String::new();
    file.read_to_string(&mut result)?;
    Ok(result
        .split('\n')
        .filter_map(|x| {
            if x.is_empty() {
                None
            } else {
                Some(x.to_string())
            }
        })
        .collect())
}

/// show metadata of the file/dirctory in `path`
pub fn metadata(path: &str) -> AxResult<Stat> {
    let mut file = File::open(path)?;
    let ret = file.stat()?;
    Ok(ret)
}

static CURRENT_DIR_PATH: Mutex<String> = Mutex::new(String::new());
/// Inspection and manipulation of the process’s environment.
pub mod env {
    use axerrno::{ax_err, AxError, AxResult};
    extern crate alloc;
    use alloc::string::{String, ToString};

    use super::{scheme_helper, CURRENT_DIR_PATH};

    pub(crate) fn canonicalize(path: &str) -> AxResult<String> {
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

    pub(crate) fn absolute_path(path: &str) -> AxResult<String> {
        let res = if path.starts_with('/') {
            canonicalize(path)
        } else {
            let mut old = CURRENT_DIR_PATH.lock().clone();
            old.push('/');
            old.push_str(path);
            canonicalize(&old)
        }?;
        if res.is_empty() {
            Ok("/".to_string())
        } else {
            Ok(res)
        }
    }
}

pub(crate) fn init() {
    *CURRENT_DIR_PATH.lock() = "/".to_string();
}

mod scheme_helper {
    pub(crate) fn get_scheme(url: &str) -> Option<(&str, &str)> {
        let mut url = url.splitn(2, ':');
        match (url.next(), url.next()) {
            (Some(scheme), Some(path)) => Some((scheme, path)),
            (Some(path), None) => Some(("file", path)),
            _ => None,
        }
    }
}

pub(crate) fn op_wrapper<F>(url: &str, f: F) -> AxResult<usize>
where
    F: FnOnce(&str) -> isize,
{
    let (scheme, path) = scheme_helper::get_scheme(url).ok_or(AxError::InvalidInput)?;
    info!("URL {}, {}", scheme, path);
    if scheme == "file" {
        let absolute = absolute_path(path)?;
        info!("     -> {}", absolute);
        from_ret_code(f(&absolute))
    } else {
        from_ret_code(f(url))
    }
}

pub(crate) fn open_wrapper(url: &str, flags: OpenFlags) -> AxResult<usize> {
    op_wrapper(url, |path| open(path, flags))
}
