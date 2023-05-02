use alloc::sync::Arc;
use axerrno::{LinuxError, LinuxResult};
use core::ffi::{c_char, c_int, c_void};

use super::{ctypes, utils::char_ptr_to_str};
use crate::debug;
use crate::fs::{File, OpenOptions};
use crate::io::{self, prelude::*, SeekFrom};
use crate::sync::Mutex;

const FILE_LIMIT: usize = 256;
const FD_NONE: Option<Arc<Mutex<File>>> = None;

/// File Descriptor Table
static FD_TABLE: Mutex<[Option<Arc<Mutex<File>>>; FILE_LIMIT]> = Mutex::new([FD_NONE; FILE_LIMIT]);

/// Get the [`File`] structure from `FD_TABLE` by `fd`.
fn get_file_by_fd(fd: c_int) -> LinuxResult<Arc<Mutex<File>>> {
    FD_TABLE
        .lock()
        .get(fd as usize)
        .and_then(|f| f.as_ref())
        .cloned()
        .ok_or(LinuxError::EBADF)
}

/// Add a new file into `FD_TABLE` and return its fd.
fn add_new_fd(file: File) -> Option<usize> {
    let mut fd_table = FD_TABLE.lock();
    for fd in 3..FILE_LIMIT {
        if fd_table[fd].is_none() {
            fd_table[fd] = Some(Arc::new(Mutex::new(file)));
            return Some(fd);
        }
    }
    None
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
/// Return its index in the file table (`fd`). Return `ENFILE` if the file
/// table overflows.
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
        add_new_fd(file).ok_or(LinuxError::ENFILE)
    })
}

/// Close a file by `fd`.
#[no_mangle]
pub unsafe extern "C" fn ax_close(fd: c_int) -> c_int {
    debug!("ax_close <= {}", fd);
    if (0..2).contains(&fd) {
        return 0; // stdin, stdout, stderr
    }
    ax_call_body!(ax_close, {
        FD_TABLE
            .lock()
            .get_mut(fd as usize)
            .and_then(|file| file.take())
            .ok_or(LinuxError::EBADF)?;
        Ok(0)
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
        let file = get_file_by_fd(fd)?;
        let off = file.lock().seek(pos)?;
        Ok(off)
    })
}

/// Read data from the file indicated by `fd`.
///
/// Return the read size if success.
#[no_mangle]
pub unsafe extern "C" fn ax_read(fd: c_int, buf: *mut c_void, count: usize) -> ctypes::ssize_t {
    debug!("ax_read <= {} {:#x} {}", fd, buf as usize, count);
    ax_call_body!(ax_read, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count) };
        let file = get_file_by_fd(fd)?;
        let len = file.lock().read(dst)?;
        Ok(len)
    })
}

/// Write data to the file indicated by `fd`.
///
/// Return the written size if success.
#[no_mangle]
pub unsafe extern "C" fn ax_write(fd: c_int, buf: *const c_void, count: usize) -> ctypes::ssize_t {
    debug!("ax_write <= {} {:#x} {}", fd, buf as usize, count);
    ax_call_body!(ax_write, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let src = unsafe { core::slice::from_raw_parts(buf as *const u8, count) };
        let file = get_file_by_fd(fd)?;
        let len = file.lock().write(src)?;
        Ok(len)
    })
}

fn stat_file(file: &File) -> io::Result<ctypes::stat> {
    let metadata = file.metadata()?;
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
        let file = File::open(path?)?;
        let st = stat_file(&file)?;
        drop(file);
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

/// Get file metadata by `fd` and write into `buf`.
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn ax_fstat(fd: c_int, buf: *mut ctypes::stat) -> ctypes::ssize_t {
    debug!("ax_fstat <= {} {:#x}", fd, buf as usize);
    ax_call_body!(ax_fstat, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let file = get_file_by_fd(fd)?;
        let st = stat_file(&file.lock())?;
        unsafe { *buf = st };
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
