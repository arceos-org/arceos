use alloc::sync::Arc;
use axerrno::{LinuxError, LinuxResult};
use core::ffi::{c_char, c_int};

use super::{ctypes, fd_ops::FileLike, utils::char_ptr_to_str};
use crate::fs::OpenOptions;
use crate::io::{prelude::*, SeekFrom};
use crate::sync::Mutex;

pub struct File(Mutex<crate::fs::File>);

impl File {
    fn new(inner: crate::fs::File) -> Self {
        Self(Mutex::new(inner))
    }

    fn add_to_fd_table(self) -> LinuxResult<c_int> {
        super::fd_ops::add_file_like(Arc::new(self))
    }

    fn from_fd(fd: c_int) -> LinuxResult<Arc<Self>> {
        let f = super::fd_ops::get_file_like(fd)?;
        f.into_any()
            .downcast::<Self>()
            .map_err(|_| LinuxError::EINVAL)
    }
}

impl FileLike for File {
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        let len = self.0.lock().read(buf)?;
        Ok(len)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        let len = self.0.lock().write(buf)?;
        Ok(len)
    }

    fn stat(&self) -> LinuxResult<ctypes::stat> {
        let metadata = self.0.lock().metadata()?;
        let metadata = metadata.raw_metadata();
        let ty = metadata.file_type() as u8;
        let perm = metadata.perm().bits() as u32;
        let st_mode = ((ty as u32) << 12) | perm;
        Ok(ctypes::stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_size: metadata.size() as _,
            st_blocks: metadata.blocks() as _,
            st_blksize: 512,
            ..Default::default()
        })
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }
}

/// Convert open flags to [`OpenOptions`].
fn flags_to_options(flags: c_int, _mode: ctypes::mode_t) -> OpenOptions {
    let flags = flags as u32;
    let mut options = OpenOptions::new();
    match flags & 0b11 {
        ctypes::O_RDONLY => options.read(true),
        ctypes::O_WRONLY => options.write(true),
        _ => options.read(true).write(true),
    };
    if flags & ctypes::O_APPEND != 0 {
        options.append(true);
    }
    if flags & ctypes::O_TRUNC != 0 {
        options.truncate(true);
    }
    if flags & ctypes::O_CREAT != 0 {
        options.create(true);
    }
    if flags & ctypes::O_EXEC != 0 {
        options.create_new(true);
    }
    options
}

/// Open a file by `filename` and insert it into the file descriptor table.
///
/// Return its index in the file table (`fd`). Return `EMFILE` if it already
/// has the maximum number of files open.
#[no_mangle]
pub unsafe extern "C" fn ax_open(
    filename: *const c_char,
    flags: c_int,
    mode: ctypes::mode_t,
) -> c_int {
    let filename = char_ptr_to_str(filename);
    debug!("ax_open <= {:?} {:#o} {:#o}", filename, flags, mode);
    ax_call_body!(ax_open, {
        let options = flags_to_options(flags, mode);
        let file = options.open(filename?)?;
        File::new(file).add_to_fd_table()
    })
}

/// Set the position of the file indicated by `fd`.
///
/// Return its position after seek.
#[no_mangle]
pub unsafe extern "C" fn ax_lseek(
    fd: c_int,
    offset: ctypes::off_t,
    whence: c_int,
) -> ctypes::off_t {
    debug!("ax_lseek <= {} {} {}", fd, offset, whence);
    ax_call_body!(ax_lseek, {
        let pos = match whence {
            0 => SeekFrom::Start(offset as _),
            1 => SeekFrom::Current(offset as _),
            2 => SeekFrom::End(offset as _),
            _ => return Err(LinuxError::EINVAL),
        };
        let off = File::from_fd(fd)?.0.lock().seek(pos)?;
        Ok(off)
    })
}

/// Get the file metadata by `path` and write into `buf`.
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn ax_stat(path: *const c_char, buf: *mut ctypes::stat) -> ctypes::ssize_t {
    let path = char_ptr_to_str(path);
    debug!("ax_stat <= {:?} {:#x}", path, buf as usize);
    ax_call_body!(ax_stat, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let file = crate::fs::File::open(path?)?;
        let st = File::new(file).stat()?;
        unsafe { *buf = st };
        Ok(0)
    })
}

/// Get the metadata of the symbolic link and write into `buf`.
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn ax_lstat(path: *const c_char, buf: *mut ctypes::stat) -> ctypes::ssize_t {
    let path = char_ptr_to_str(path);
    debug!("ax_lstat <= {:?} {:#x}", path, buf as usize);
    ax_call_body!(ax_lstat, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        unsafe { *buf = Default::default() }; // TODO
        Ok(0)
    })
}

/// Get the path of the current directory.
#[no_mangle]
pub unsafe extern "C" fn ax_getcwd(buf: *mut c_char, size: usize) -> *mut c_char {
    debug!("ax_getcwd <= {:#x} {}", buf as usize, size);
    ax_call_body!(ax_getcwd, {
        if buf.is_null() {
            return Ok(core::ptr::null::<c_char>() as _);
        }
        let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, size as _) };
        let cwd = crate::env::current_dir()?;
        let cwd = cwd.as_bytes();
        if cwd.len() < size {
            dst[..cwd.len()].copy_from_slice(cwd);
            dst[cwd.len()] = 0;
            Ok(buf)
        } else {
            Err(LinuxError::ERANGE)
        }
    })
}
