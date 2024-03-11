// const STDIN: usize = 0;
// const STDOUT: usize = 1;
// const STDERR: usize = 2;
extern crate alloc;

use crate::{SyscallError, SyscallResult};
use axlog::debug;
use axprocess::link::{create_link, deal_with_path, remove_link, FilePath};

/// Special value used to indicate openat should use the current working directory.
pub const AT_REMOVEDIR: usize = 0x200; // Remove directory instead of unlinking file.

/// 功能:创建文件的链接；
/// # Arguments
/// * `old_dir_fd`: usize, 原来的文件所在目录的文件描述符。
/// * `old_path`: *const u8, 文件原来的名字。如果old_path是相对路径,则它是相对于old_dir_fd目录而言的。如果old_path是相对路径,且old_dir_fd的值为AT_FDCWD,则它是相对于当前路径而言的。如果old_path是绝对路径,则old_dir_fd被忽略。
/// * `new_dir_fd`: usize, 新文件名所在的目录。
/// * `new_path`: *const u8, 文件的新名字。new_path的使用规则同old_path。
/// * `flags`: usize, 在2.6.18内核之前,应置为0。其它的值详见`man 2 linkat`。
/// # Return
/// 成功执行,返回0。失败,返回-1。
#[allow(dead_code)]
pub fn sys_linkat(args: [usize; 6]) -> SyscallResult {
    let old_dir_fd = args[0];
    let old_path = args[1] as *const u8;
    let new_dir_fd = args[2];
    let new_path = args[3] as *const u8;
    let _flags = args[4];

    let old_path = if let Some(path) = deal_with_path(old_dir_fd, Some(old_path), false) {
        path
    } else {
        return Err(SyscallError::EINVAL);
    };
    let new_path = if let Some(path) = deal_with_path(new_dir_fd, Some(new_path), false) {
        path
    } else {
        return Err(SyscallError::EINVAL);
    };
    if create_link(&old_path, &new_path) {
        Ok(0)
    } else {
        Err(SyscallError::EINVAL)
    }
}

/// 功能:移除指定文件的链接
/// # Arguments
/// * `path`: *const u8, 要删除的链接的名字。
/// # Return
/// 成功执行,返回0。失败,返回-1。
pub fn syscall_unlink(args: [usize; 6]) -> SyscallResult {
    let path = args[0] as *const u8;
    let temp_args = [axprocess::link::AT_FDCWD, path as usize, 0, 0, 0, 0];
    syscall_unlinkat(temp_args)
}

/// 功能:移除指定文件的链接(可用于删除文件);
/// # Arguments
/// * `dir_fd`: usize, 要删除的链接所在的目录。
/// * `path`: *const u8, 要删除的链接的名字。如果path是相对路径,则它是相对于dir_fd目录而言的。如果path是相对路径,且dir_fd的值为AT_FDCWD,则它是相对于当前路径而言的。如果path是绝对路径,则dir_fd被忽略。
/// * `flags`: usize, 可设置为0或AT_REMOVEDIR。
/// # Return
/// 成功执行,返回0。失败,返回-1。
pub fn syscall_unlinkat(args: [usize; 6]) -> SyscallResult {
    let dir_fd = args[0];
    let path = args[1] as *const u8;
    let flags = args[2];
    let path = deal_with_path(dir_fd, Some(path), false).unwrap();

    if path.start_with(&FilePath::new("/proc").unwrap()) {
        return Ok(-1);
    }

    // unlink file
    if flags == 0 {
        if remove_link(&path).is_none() {
            debug!("unlink file error");
            return Err(SyscallError::EINVAL);
        }
    }
    // remove dir
    else if flags == AT_REMOVEDIR {
        if let Err(e) = axfs::api::remove_dir(path.path()) {
            debug!("rmdir error: {:?}", e);
            return Err(SyscallError::EINVAL);
        }
    }
    // flags error
    else {
        debug!("flags error");
        return Err(SyscallError::EINVAL);
    }
    Ok(0)
}
