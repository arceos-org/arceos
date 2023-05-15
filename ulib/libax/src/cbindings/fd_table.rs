#[allow(unused_imports)]
use alloc::sync::Arc;
use axerrno::{LinuxError, LinuxResult};
use core::ffi::{c_int, c_void};

use super::ctypes;
use crate::debug;
use crate::sync::Mutex;

#[cfg(feature = "net")]
use super::socket::Socket;
#[cfg(feature = "fs")]
use {crate::fs::File, crate::io::prelude::*};

const FILE_LIMIT: usize = 1024;
const FD_NONE: Option<Filelike> = None;

#[derive(Clone)]
pub enum Filelike {
    #[cfg(feature = "fs")]
    FileType(Arc<Mutex<File>>),
    #[cfg(feature = "net")]
    SocketType(Arc<Mutex<Socket>>),
}

/// File Descriptor Table
static FD_TABLE: Mutex<[Option<Filelike>; FILE_LIMIT]> = Mutex::new([FD_NONE; FILE_LIMIT]);

impl Filelike {
    pub fn from_fd(fd: c_int) -> LinuxResult<Self> {
        FD_TABLE
            .lock()
            .get(fd as usize)
            .and_then(|f| f.as_ref())
            .cloned()
            .ok_or(LinuxError::EBADF)
    }

    #[cfg(feature = "net")]
    #[allow(irrefutable_let_patterns)]
    pub fn into_socket(self) -> LinuxResult<Arc<Mutex<Socket>>> {
        if let Filelike::SocketType(s) = self {
            Ok(s)
        } else {
            Err(LinuxError::ENOTSOCK)
        }
    }

    #[cfg(feature = "fs")]
    #[allow(irrefutable_let_patterns)]
    pub fn into_file(self) -> LinuxResult<Arc<Mutex<File>>> {
        if let Filelike::FileType(s) = self {
            Ok(s)
        } else {
            Err(LinuxError::ESPIPE)
        }
    }

    pub fn stat(&self) -> LinuxResult<ctypes::stat> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "fs")]
            Filelike::FileType(f) => Ok(super::file::stat_file(&f.lock())?),
            #[cfg(feature = "net")]
            Filelike::SocketType(s) => Ok(super::socket::stat_socket(&s.lock())?),
            _ => Err(LinuxError::EINVAL),
        }
    }

    pub fn write(&self, _src: &[u8]) -> LinuxResult<ctypes::ssize_t> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "fs")]
            Filelike::FileType(f) => {
                let len = f.lock().write(_src)?;
                Ok(len as isize)
            }
            #[cfg(feature = "net")]
            Filelike::SocketType(s) => {
                let len = s.lock().send(_src)?;
                Ok(len as isize)
            }
            _ => Err(LinuxError::EINVAL),
        }
    }

    pub fn read(&self, _dst: &mut [u8]) -> LinuxResult<ctypes::ssize_t> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "fs")]
            Filelike::FileType(f) => {
                let len = f.lock().read(_dst)?;
                Ok(len as isize)
            }
            #[cfg(feature = "net")]
            Filelike::SocketType(s) => {
                let len = s.lock().recv(_dst)?;
                Ok(len as isize)
            }
            _ => Err(LinuxError::EINVAL),
        }
    }
}

/// Add a new file into `FD_TABLE` and return its fd.
#[cfg(feature = "fs")]
pub fn add_new_file(file: File) -> Option<usize> {
    let mut fd_table = FD_TABLE.lock();
    for fd in 3..FILE_LIMIT {
        if fd_table[fd].is_none() {
            fd_table[fd] = Some(Filelike::FileType(Arc::new(Mutex::new(file))));
            return Some(fd);
        }
    }
    None
}

/// Add a new socket into `FD_TABLE` and return its fd.
#[cfg(feature = "net")]
pub fn add_new_socket(socket: Socket) -> Option<usize> {
    let mut fd_table = FD_TABLE.lock();
    for fd in 3..FILE_LIMIT {
        if fd_table[fd].is_none() {
            fd_table[fd] = Some(Filelike::SocketType(Arc::new(Mutex::new(socket))));
            return Some(fd);
        }
    }
    None
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
        Filelike::from_fd(fd)?.read(dst)
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
        Filelike::from_fd(fd)?.write(src)
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
        unsafe { *buf = Filelike::from_fd(fd)?.stat()? };
        Ok(0)
    })
}
