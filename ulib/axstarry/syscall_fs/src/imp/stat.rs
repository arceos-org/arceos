//! 获取文件系统状态信息
//!

use axfs::api::{FileIOType, Kstat};
use axlog::{debug, error, info};
use axprocess::{
    current_process,
    link::{deal_with_path, raw_ptr_to_ref_str, FilePath, AT_FDCWD},
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
    let file_path = if let Some(file_path) = deal_with_path(dir_fd, Some(path), false) {
        file_path
    } else {
        // x86 下应用会调用 newfstatat(1, "", {st_mode=S_IFCHR|0620, st_rdev=makedev(0x88, 0xe), ...}, AT_EMPTY_PATH) = 0
        // 去尝试检查 STDOUT 的属性。这里暂时先特判，以后再改成真正的 stdout 的属性
        let path = unsafe { raw_ptr_to_ref_str(path) };
        if path.len() == 0 && dir_fd == 1 {
            unsafe {
                (*kst).st_mode = 0o20000 | 0o220u32;
                (*kst).st_ino = 1;
                (*kst).st_nlink = 1;
            }
            return Ok(0)
        }
        panic!("Wrong path at syscall_fstatat: {}(dir_fd={})", path, dir_fd);
    };
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

/// 获取文件状态信息
pub fn syscall_lstat(path: *const u8, kst: *mut Kstat) -> SyscallResult {
    syscall_fstatat(AT_FDCWD, path, kst)
}

/// 获取文件状态信息
pub fn syscall_stat(path: *const u8, stat_ptr: *mut Kstat) -> SyscallResult {
    syscall_fstatat(AT_FDCWD, path, stat_ptr)
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
