#[allow(unused_imports)]
use alloc::sync::Arc;
use axerrno::{LinuxError, LinuxResult};
use core::ffi::{c_int, c_void};

use super::ctypes;
use crate::sync::Mutex;
use crate::{debug, warn};

#[cfg(feature = "pipe")]
use super::pipe::Pipe;
#[cfg(feature = "net")]
use super::socket::Socket;
#[cfg(feature = "fs")]
use {crate::fs::File, crate::io::prelude::*};

pub const AX_FILE_LIMIT: usize = 1024;
const FD_NONE: Option<Filelike> = None;

/// Unified fd enumerator, second parameter corresponds to `flags` used by fcntl
#[allow(clippy::enum_variant_names)]
#[derive(Clone)]
pub enum Filelike {
    #[cfg(feature = "fs")]
    FileType(Arc<Mutex<File>>),
    #[cfg(feature = "net")]
    SocketType(Arc<Mutex<Socket>>),
    #[cfg(feature = "pipe")]
    PipeType(Arc<Mutex<Pipe>>),
}

/// File Descriptor Table
static FD_TABLE: Mutex<[Option<Filelike>; AX_FILE_LIMIT]> = Mutex::new([FD_NONE; AX_FILE_LIMIT]);

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
    pub fn from_socket(socket: Socket) -> Self {
        Self::SocketType(Arc::new(Mutex::new(socket)))
    }

    #[cfg(feature = "fs")]
    pub fn from_file(file: File) -> Self {
        Self::FileType(Arc::new(Mutex::new(file)))
    }

    #[cfg(feature = "pipe")]
    pub fn from_pipe(pipe: Arc<Mutex<Pipe>>) -> Self {
        Self::PipeType(pipe)
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

    #[cfg(feature = "pipe")]
    #[allow(dead_code)]
    #[allow(irrefutable_let_patterns)]
    pub fn into_pipe(self) -> LinuxResult<Arc<Mutex<Pipe>>> {
        if let Filelike::PipeType(p) = self {
            Ok(p)
        } else {
            Err(LinuxError::ESPIPE)
        }
    }

    #[allow(dead_code)]
    pub fn add_to_fd_table(self) -> Option<usize> {
        let mut fd_table = FD_TABLE.lock();
        for fd in 3..AX_FILE_LIMIT {
            if fd_table[fd].is_none() {
                fd_table[fd] = Some(self);
                return Some(fd);
            }
        }
        None
    }

    pub fn stat(&self) -> LinuxResult<ctypes::stat> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "fs")]
            Filelike::FileType(f) => Ok(super::file::stat_file(&f.lock())?),
            #[cfg(feature = "net")]
            Filelike::SocketType(s) => Ok(super::socket::stat_socket(&s.lock())?),
            #[cfg(feature = "pipe")]
            Filelike::PipeType(p) => Ok(super::pipe::stat_pipe(&p.lock())?),
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
            #[cfg(feature = "pipe")]
            Filelike::PipeType(p) => {
                let len = p.lock().write(_src)?;
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
            #[cfg(feature = "pipe")]
            Filelike::PipeType(p) => {
                let len = p.lock().read(_dst)?;
                Ok(len as isize)
            }
            _ => Err(LinuxError::EINVAL),
        }
    }
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

fn dup_fd(old_fd: c_int) -> LinuxResult<c_int> {
    let mut fd_table = FD_TABLE.lock();
    if fd_table[old_fd as usize].is_none() {
        return Err(LinuxError::EBADF);
    }
    for fd in 3..AX_FILE_LIMIT {
        if fd_table[fd].is_none() {
            fd_table[fd] = fd_table[old_fd as usize].clone();
            return Ok(fd as c_int);
        }
    }
    Err(LinuxError::ENFILE)
}

/// Duplicate a file descriptor
#[no_mangle]
pub unsafe extern "C" fn ax_dup(old_fd: c_int) -> c_int {
    debug!("ax_dup <= {}", old_fd);
    ax_call_body!(ax_dup, dup_fd(old_fd))
}

/// `dup3()` is the same as `dup2()`, except that:
///
/// The caller can force the close-on-exec flag to be set for the new file descriptor by specifying `O_CLOEXEC` in flags.  
///
/// If oldfd equals newfd, then `dup3()` fails with the error `EINVAL`.
#[no_mangle]
pub unsafe extern "C" fn ax_dup3(old_fd: c_int, new_fd: c_int, flags: c_int) -> c_int {
    debug!(
        "ax_dup3 <= old_fd: {}, new_fd: {}, flags: {}",
        old_fd, new_fd, flags
    );

    ax_call_body!(ax_dup3, {
        if old_fd == new_fd {
            return Err(LinuxError::EINVAL);
        }
        if new_fd as usize >= AX_FILE_LIMIT {
            return Err(LinuxError::ENFILE);
        }

        let mut fd_table = FD_TABLE.lock();
        if fd_table[old_fd as usize].is_none() {
            return Err(LinuxError::EBADF);
        }
        fd_table[new_fd as usize].take();
        fd_table[new_fd as usize] = fd_table[old_fd as usize].clone();
        drop(fd_table);

        if flags as u32 & ctypes::O_CLOEXEC != 0 {
            ax_fcntl(
                new_fd,
                ctypes::F_SETFD as c_int,
                ctypes::FD_CLOEXEC as usize,
            );
        }
        Ok(new_fd)
    })
}

/// Fcntl implementation
///
/// TODO: `SET/GET` command is ignored
#[no_mangle]
pub unsafe extern "C" fn ax_fcntl(fd: c_int, cmd: c_int, arg: usize) -> c_int {
    debug!("ax_fcntl <= fd: {} cmd: {} arg: {}", fd, cmd, arg);
    ax_call_body!(ax_fcntl, {
        match cmd as u32 {
            ctypes::F_DUPFD => dup_fd(fd),
            ctypes::F_DUPFD_CLOEXEC => {
                // TODO: Change fd flags
                dup_fd(fd)
            }
            _ => {
                warn!("unsupported fcntl parameters: cmd {}", cmd);
                Ok(0)
            }
        }
    })
}
