use alloc::sync::Arc;
use core::ffi::c_int;

use axerrno::{LinuxError, LinuxResult};
use axio::PollState;
use axns::{ResArc, def_resource};
use flatten_objects::FlattenObjects;
use spin::RwLock;

use crate::ctypes;
use crate::imp::stdio::{stdin, stdout};

pub const AX_FILE_LIMIT: usize = 1024;

#[allow(dead_code)]
pub trait FileLike: Send + Sync {
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize>;
    fn write(&self, buf: &[u8]) -> LinuxResult<usize>;
    fn stat(&self) -> LinuxResult<ctypes::stat>;
    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync>;
    fn poll(&self) -> LinuxResult<PollState>;
    fn set_nonblocking(&self, nonblocking: bool) -> LinuxResult;
}

def_resource! {
    pub static FD_TABLE: ResArc<RwLock<FlattenObjects<Arc<dyn FileLike>, AX_FILE_LIMIT>>> = ResArc::new();
}

impl FD_TABLE {
    /// Return a copy of the inner table.
    pub fn copy_inner(&self) -> RwLock<FlattenObjects<Arc<dyn FileLike>, AX_FILE_LIMIT>> {
        let table = self.read();
        let mut new_table = FlattenObjects::new();
        for id in table.ids() {
            let _ = new_table.add_at(id, table.get(id).unwrap().clone());
        }
        RwLock::new(new_table)
    }
}

/// Get a file by `fd`.
pub fn get_file_like(fd: c_int) -> LinuxResult<Arc<dyn FileLike>> {
    FD_TABLE
        .read()
        .get(fd as usize)
        .cloned()
        .ok_or(LinuxError::EBADF)
}

/// Add a file to the file descriptor table.
pub fn add_file_like(f: Arc<dyn FileLike>) -> LinuxResult<c_int> {
    Ok(FD_TABLE.write().add(f).map_err(|_| LinuxError::EMFILE)? as c_int)
}

/// Close a file by `fd`.
pub fn close_file_like(fd: c_int) -> LinuxResult {
    let f = FD_TABLE
        .write()
        .remove(fd as usize)
        .ok_or(LinuxError::EBADF)?;
    drop(f);
    Ok(())
}

/// Close a file by `fd`.
pub fn sys_close(fd: c_int) -> c_int {
    debug!("sys_close <= {}", fd);
    if (0..=2).contains(&fd) {
        return 0; // stdin, stdout, stderr
    }
    syscall_body!(sys_close, close_file_like(fd).map(|_| 0))
}

fn dup_fd(old_fd: c_int) -> LinuxResult<c_int> {
    let f = get_file_like(old_fd)?;
    let new_fd = add_file_like(f)?;
    Ok(new_fd)
}

/// Duplicate a file descriptor.
pub fn sys_dup(old_fd: c_int) -> c_int {
    debug!("sys_dup <= {}", old_fd);
    syscall_body!(sys_dup, dup_fd(old_fd))
}

/// Duplicate a file descriptor, but it uses the file descriptor number specified in `new_fd`.
///
/// TODO: `dup2` should forcibly close new_fd if it is already opened.
pub fn sys_dup2(old_fd: c_int, new_fd: c_int) -> c_int {
    debug!("sys_dup2 <= old_fd: {}, new_fd: {}", old_fd, new_fd);
    syscall_body!(sys_dup2, {
        if old_fd == new_fd {
            let r = sys_fcntl(old_fd, ctypes::F_GETFD as _, 0);
            if r >= 0 {
                return Ok(old_fd);
            } else {
                return Ok(r);
            }
        }
        if new_fd as usize >= AX_FILE_LIMIT {
            return Err(LinuxError::EBADF);
        }

        let f = get_file_like(old_fd)?;
        FD_TABLE
            .write()
            .add_at(new_fd as usize, f)
            .map_err(|_| LinuxError::EMFILE)?;

        Ok(new_fd)
    })
}

/// Manipulate file descriptor.
///
/// TODO: `SET/GET` command is ignored, hard-code stdin/stdout
pub fn sys_fcntl(fd: c_int, cmd: c_int, arg: usize) -> c_int {
    debug!("sys_fcntl <= fd: {} cmd: {} arg: {}", fd, cmd, arg);
    syscall_body!(sys_fcntl, {
        match cmd as u32 {
            ctypes::F_DUPFD => dup_fd(fd),
            ctypes::F_DUPFD_CLOEXEC => {
                // TODO: Change fd flags
                dup_fd(fd)
            }
            ctypes::F_SETFL => {
                if fd == 0 || fd == 1 || fd == 2 {
                    return Ok(0);
                }
                get_file_like(fd)?.set_nonblocking(arg & (ctypes::O_NONBLOCK as usize) > 0)?;
                Ok(0)
            }
            _ => {
                warn!("unsupported fcntl parameters: cmd {}", cmd);
                Ok(0)
            }
        }
    })
}

#[ctor_bare::register_ctor]
fn init_stdio() {
    let mut fd_table = flatten_objects::FlattenObjects::new();
    fd_table
        .add_at(0, Arc::new(stdin()) as _)
        .unwrap_or_else(|_| panic!()); // stdin
    fd_table
        .add_at(1, Arc::new(stdout()) as _)
        .unwrap_or_else(|_| panic!()); // stdout
    fd_table
        .add_at(2, Arc::new(stdout()) as _)
        .unwrap_or_else(|_| panic!()); // stderr
    FD_TABLE.init_new(spin::RwLock::new(fd_table));
}
