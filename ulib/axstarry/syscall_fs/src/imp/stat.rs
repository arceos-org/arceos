//! 获取文件系统状态信息
//!

use axfs::api::{FileIOType, Kstat};
use axlog::{debug, error, info};
use axprocess::{
    current_process,
    link::{deal_with_path, FilePath, AT_FDCWD},
};
use syscall_utils::{get_fs_stat, FsStat, SyscallError, SyscallResult};

use crate::ctype::mount::get_stat_in_fs;

/// 实现 stat 系列系统调用
pub fn syscall_fstat(fd: usize, kst: *mut Kstat) -> SyscallResult {
    let process = current_process();
    let fd_table = process.fd_manager.fd_table.lock();

    if fd >= fd_table.len() || fd < 3 {
        debug!("fd {} is out of range", fd);
        return Err(SyscallError::EPERM);
    }
    if fd_table[fd].is_none() {
        debug!("fd {} is none", fd);
        return Err(SyscallError::EPERM);
    }
    let file = fd_table[fd].clone().unwrap();
    if file.get_type() != FileIOType::FileDesc {
        debug!("fd {} is not a file", fd);
        return Err(SyscallError::EPERM);
    }

    match file.get_stat() {
        Ok(stat) => {
            unsafe {
                *kst = stat;
            }
            Ok(0)
        }
        Err(e) => {
            debug!("get stat error: {:?}", e);
            Err(SyscallError::EPERM)
        }
    }
}

/// 获取文件状态信息，但是给出的是目录 fd 和相对路径。
pub fn syscall_fstatat(dir_fd: usize, path: *const u8, kst: *mut Kstat) -> SyscallResult {
    let file_path = deal_with_path(dir_fd, Some(path), false).unwrap();
    info!("path : {}", file_path.path());
    match get_stat_in_fs(&file_path) {
        Ok(stat) => unsafe {
            *kst = stat;
            Ok(0)
        },
        Err(error_no) => {
            debug!("get stat error: {:?}", error_no);
            Err(error_no)
        }
    }
}

/// 获取文件系统的信息
pub fn syscall_statfs(path: *const u8, stat: *mut FsStat) -> SyscallResult {
    let file_path = deal_with_path(AT_FDCWD, Some(path), false).unwrap();
    if file_path.equal_to(&FilePath::new("/").unwrap()) {
        // 目前只支持访问根目录文件系统的信息
        unsafe {
            *stat = get_fs_stat();
        }

        Ok(0)
    } else {
        error!("Only support fs_stat for root");
        Err(SyscallError::EINVAL)
    }
}
