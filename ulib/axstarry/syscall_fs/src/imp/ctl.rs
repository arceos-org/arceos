//! 对文件系统的管理，包括目录项的创建、文件权限设置等内容
use axfs::api::{OpenFlags, Permissions};
use axlog::{debug, error, info};
use core::{mem::transmute, ptr::copy_nonoverlapping};

use axhal::mem::VirtAddr;
use axprocess::{
    current_process,
    link::{deal_with_path, FilePath, AT_FDCWD},
};
use syscall_utils::{DirEnt, DirEntType, Fcntl64Cmd, SyscallError, SyscallResult, TimeSecs};

use crate::{ctype::file::new_fd, FileDesc};

extern crate alloc;
use alloc::string::ToString;

/// 功能：获取当前工作目录；
/// 输入：
///     - char *buf：一块缓存区，用于保存当前工作目录的字符串。当buf设为NULL，由系统来分配缓存区。
///     - size：buf缓存区的大小。
/// 返回值：成功执行，则返回当前工作目录的字符串的指针。失败，则返回NULL。
///  暂时：成功执行，则返回当前工作目录的字符串的指针 as isize。失败，则返回0。
///
/// 注意：当前写法存在问题，cwd应当是各个进程独立的，而这里修改的是整个fs的目录
pub fn syscall_getcwd(buf: *mut u8, len: usize) -> SyscallResult {
    debug!("Into syscall_getcwd. buf: {}, len: {}", buf as usize, len);
    let cwd = axfs::api::current_dir().unwrap();

    // todo: 如果buf为NULL,则系统分配缓存区
    // let process = current_process();
    // let process_inner = process.inner.lock();
    // if buf.is_null() {
    //     buf = allocate_buffer(cwd.len());   // 分配缓存区 allocate_buffer
    // }

    let cwd = cwd.as_bytes();

    return if len >= cwd.len() {
        let process = current_process();
        let start: VirtAddr = (buf as usize).into();
        let end = start + len;
        if process.manual_alloc_range_for_lazy(start, end).is_ok() {
            unsafe {
                core::ptr::copy_nonoverlapping(cwd.as_ptr(), buf, cwd.len());
            }
            Ok(buf as isize)
        } else {
            // ErrorNo::EINVAL as isize
            Err(SyscallError::EINVAL)
        }
    } else {
        debug!("getcwd: buf size is too small");
        Err(SyscallError::ERANGE)
    };
}

/// 功能：创建目录；
/// 输入：
///     - dirfd：要创建的目录所在的目录的文件描述符。
///     - path：要创建的目录的名称。如果path是相对路径，则它是相对于dirfd目录而言的。如果path是相对路径，且dirfd的值为AT_FDCWD，则它是相对于当前路径而言的。如果path是绝对路径，则dirfd被忽略。
///     - mode：文件的所有权描述。详见`man 7 inode `。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_mkdirat(dir_fd: usize, path: *const u8, mode: u32) -> SyscallResult {
    // info!("signal module: {:?}", process_inner.signal_module.keys());
    let path = if let Some(path) = deal_with_path(dir_fd, Some(path), true) {
        path
    } else {
        return Err(SyscallError::EINVAL);
    };
    debug!(
        "Into syscall_mkdirat. dirfd: {}, path: {:?}, mode: {}",
        dir_fd,
        path.path(),
        mode
    );
    if axfs::api::path_exists(path.path()) {
        // 文件已存在
        return Err(SyscallError::EEXIST);
    }
    let _ = axfs::api::create_dir(path.path());
    // 只要文件夹存在就返回0
    if axfs::api::path_exists(path.path()) {
        Ok(0)
    } else {
        Err(SyscallError::EPERM)
    }
}

/// 功能：切换工作目录；
/// 输入：
///     - path：需要切换到的目录。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_chdir(path: *const u8) -> SyscallResult {
    // 从path中读取字符串
    let path = if let Some(path) = deal_with_path(AT_FDCWD, Some(path), true) {
        path
    } else {
        return Err(SyscallError::EINVAL);
    };
    debug!("Into syscall_chdir. path: {:?}", path.path());
    match axfs::api::set_current_dir(path.path()) {
        Ok(_) => Ok(0),
        Err(_) => Err(SyscallError::EINVAL),
    }
}

/// 功能：获取目录的条目;
/// 参数：
///     -fd：所要读取目录的文件描述符。
///     -buf：一个缓存区，用于保存所读取目录的信息。缓存区的结构如下
///     -len：buf的大小。
/// 返回值：成功执行，返回读取的字节数。当到目录结尾，则返回0。失败，则返回-1。
///  struct dirent {
///      uint64 d_ino;	// 索引结点号
///      int64 d_off;	// 到下一个dirent的偏移
///      unsigned short d_reclen;	// 当前dirent的长度
///      unsigned char d_type;	// 文件类型 0:
///      char d_name[];	//文件名
///  };
///  1. 内存布局：
///       0x61fef8
///       0x61fef8 0x61ff00 0x61ff08 0x61ff0a 0x61ff0b
///       实测结果在我的电脑上是这样的，没有按最大对齐方式8字节对齐
///  2. d_off 和 d_reclen 同时存在的原因：
///       不同的dirent可以不按照顺序紧密排列
pub fn syscall_getdents64(fd: usize, buf: *mut u8, len: usize) -> SyscallResult {
    let path = if let Some(path) = deal_with_path(fd, None, true) {
        path
    } else {
        return Err(SyscallError::EINVAL);
    };

    let process = current_process();
    // 注意是否分配地址
    let start: VirtAddr = (buf as usize).into();
    let end = start + len;
    if process.manual_alloc_range_for_lazy(start, end).is_err() {
        return Err(SyscallError::EFAULT);
    }

    let entry_id_from = unsafe { (*(buf as *const DirEnt)).d_off };
    if entry_id_from == -1 {
        // 说明已经读完了
        return Ok(0);
    }

    let buf = unsafe { core::slice::from_raw_parts_mut(buf, len) };
    let dir_iter = axfs::api::read_dir(path.path()).unwrap();
    let mut count = 0; // buf中已经写入的字节数

    for (_, entry) in dir_iter.enumerate() {
        let entry = entry.unwrap();
        let mut name = entry.file_name();
        name.push('\0');
        let name = name.as_bytes();
        let name_len = name.len();
        let file_type = entry.file_type();
        let entry_size = DirEnt::fixed_size() + name_len + 1;

        // buf不够大，写不下新的entry
        if count + entry_size > len {
            debug!("buf not big enough");
            return Ok(count as isize);
        }
        // 转换为DirEnt
        let dirent: &mut DirEnt = unsafe { transmute(buf.as_mut_ptr().offset(count as isize)) };
        // 设置定长部分
        if file_type.is_dir() {
            dirent.set_fixed_part(1, entry_size as i64, entry_size, DirEntType::DIR);
        } else if file_type.is_file() {
            dirent.set_fixed_part(1, entry_size as i64, entry_size, DirEntType::REG);
        } else {
            dirent.set_fixed_part(1, entry_size as i64, entry_size, DirEntType::UNKNOWN);
        }

        // 写入文件名
        unsafe { copy_nonoverlapping(name.as_ptr(), dirent.d_name.as_mut_ptr(), name_len) };

        count += entry_size;
    }
    Ok(count as isize)
}

/// 276
/// 重命名文件或目录
// todo!
// 1. 权限检查
// 调用进程必须对源目录和目标目录都有写权限,才能完成重命名。
// 2. 目录和文件在同一个文件系统
// 如果目录和文件不在同一个文件系统,重命名会失败。renameat2不能跨文件系统重命名。
// 3. 源文件不是目标目录的子目录
// 如果源文件是目标目录的子孙目录,也会导致重命名失败。不能将目录重命名到自己的子目录中。
// 4. 目标名称不存在
// 目标文件名在目标目录下必须不存在,否则会失败。
// 5. 源文件被打开
// 如果源文件正被进程打开,默认情况下重命名也会失败。可以通过添加RENAME_EXCHANGE标志位实现原子交换。
// 6. 目录不是挂载点
// 如果源目录是一个挂载点,也不允许重命名。
pub fn syscall_renameat2(
    old_dirfd: usize,
    _old_path: *const u8,
    new_dirfd: usize,
    _new_path: *const u8,
    flags: usize,
) -> SyscallResult {
    let old_path = deal_with_path(old_dirfd, Some(_old_path), false).unwrap();
    let new_path = deal_with_path(new_dirfd, Some(_new_path), false).unwrap();

    let proc_path = FilePath::new("/proc").unwrap();
    if old_path.start_with(&proc_path) || new_path.start_with(&proc_path) {
        return Err(SyscallError::EPERM);
    }

    // 交换两个目录名，目录下的文件不受影响，

    // 如果重命名后的文件已存在
    if flags == 1 {
        // 即RENAME_NOREPLACE
        if axfs::api::path_exists(new_path.path()) {
            debug!("new_path_ already exist");
            return Err(SyscallError::EPERM);
        }
    }

    // 文件与文件夹不能互换命名
    if flags == 2 {
        // 即RENAME_EXCHANGE
        let old_metadata = axfs::api::metadata(old_path.path()).unwrap();
        let new_metadata = axfs::api::metadata(new_path.path()).unwrap();
        if old_metadata.is_dir() != new_metadata.is_dir() {
            debug!("old_path_ and new_path_ is not the same type");
            return Err(SyscallError::EPERM);
        }
    }

    if flags == 4 {
        // 即RENAME_WHITEOUT
        let new_metadata = axfs::api::metadata(new_path.path()).unwrap();
        if new_metadata.is_dir() {
            debug!("new_path_ is a directory");
            return Err(SyscallError::EPERM);
        }
    }

    if flags != 1 && flags != 2 && flags != 4 {
        debug!("flags is not valid");
        return Err(SyscallError::EPERM);
    }

    // 做实际重命名操作
    axlog::warn!("renameat2 not implemented");
    Ok(0)
}

pub fn syscall_fcntl64(fd: usize, cmd: usize, arg: usize) -> SyscallResult {
    let process = current_process();
    let mut fd_table = process.fd_manager.fd_table.lock();

    if fd >= fd_table.len() {
        debug!("fd {} is out of range", fd);
        return Err(SyscallError::EBADF);
    }
    if fd_table[fd].is_none() {
        debug!("fd {} is none", fd);
        return Err(SyscallError::EBADF);
    }
    let file = fd_table[fd].clone().unwrap();
    info!("fd: {}, cmd: {}", fd, cmd);
    match Fcntl64Cmd::try_from(cmd) {
        Ok(Fcntl64Cmd::F_DUPFD) => {
            let new_fd = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
                fd
            } else {
                // 文件描述符达到上限了
                return Err(SyscallError::EMFILE);
            };
            fd_table[new_fd] = fd_table[fd].clone();
            Ok(new_fd as isize)
        }
        Ok(Fcntl64Cmd::F_GETFD) => {
            if file.get_status().contains(OpenFlags::CLOEXEC) {
                Ok(1)
            } else {
                Ok(0)
            }
        }
        Ok(Fcntl64Cmd::F_SETFD) => {
            if file.set_close_on_exec((arg & 1) != 0) {
                Ok(0)
            } else {
                Err(SyscallError::EINVAL)
            }
        }
        Ok(Fcntl64Cmd::F_GETFL) => Ok(file.get_status().bits() as isize),
        Ok(Fcntl64Cmd::F_SETFL) => {
            if let Some(flags) = OpenFlags::from_bits(arg as u32) {
                if file.set_status(flags) {
                    return Ok(0);
                }
            }
            Err(SyscallError::EINVAL)
        }
        Ok(Fcntl64Cmd::F_DUPFD_CLOEXEC) => {
            let new_fd = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
                fd
            } else {
                // 文件描述符达到上限了
                return Err(SyscallError::EMFILE);
            };

            if file.set_close_on_exec((arg & 1) != 0) {
                fd_table[new_fd] = fd_table[fd].clone();
                Ok(new_fd as isize)
            } else {
                Err(SyscallError::EINVAL)
            }
        }
        _ => Err(SyscallError::EINVAL),
    }
}

/// 29
/// 执行各种设备相关的控制功能
/// todo: 未实现
pub fn syscall_ioctl(fd: usize, request: usize, argp: *mut usize) -> SyscallResult {
    let process = current_process();
    let fd_table = process.fd_manager.fd_table.lock();
    info!("fd: {}, request: {}, argp: {}", fd, request, argp as usize);
    if fd >= fd_table.len() {
        debug!("fd {} is out of range", fd);
        return Err(SyscallError::EBADF);
    }
    if fd_table[fd].is_none() {
        debug!("fd {} is none", fd);
        return Err(SyscallError::EBADF);
    }
    if process
        .manual_alloc_for_lazy((argp as usize).into())
        .is_err()
    {
        return Err(SyscallError::EFAULT); // 地址不合法
    }

    let file = fd_table[fd].clone().unwrap();
    // if file.lock().ioctl(request, argp as usize).is_err() {
    //     return -1;
    // }
    let _ = file.ioctl(request, argp as usize);
    Ok(0)
}

/// 53
/// 修改文件权限
/// mode: 0o777, 3位八进制数字
/// path为相对路径：
///     1. 若dir_fd为AT_FDCWD，则相对于当前工作目录
///     2. 若dir_fd为AT_FDCWD以外的值，则相对于dir_fd所指的目录
/// path为绝对路径：
///     忽视dir_fd，直接根据path访问
pub fn syscall_fchmodat(dir_fd: usize, path: *const u8, mode: usize) -> SyscallResult {
    let file_path = deal_with_path(dir_fd, Some(path), false).unwrap();
    axfs::api::metadata(file_path.path())
        .map(|mut metadata| {
            metadata.set_permissions(Permissions::from_bits_truncate(mode as u16));
            Ok(0)
        })
        .unwrap_or_else(|_| Err(SyscallError::ENOENT))
}

/// 48
/// 获取文件权限
/// 类似上面的fchmodat
///        The mode specifies the accessibility check(s) to be performed,
///        and is either the value F_OK, or a mask consisting of the bitwise
///        OR of one or more of R_OK, W_OK, and X_OK.  F_OK tests for the
///        existence of the file.  R_OK, W_OK, and X_OK test whether the
///        file exists and grants read, write, and execute permissions,
///        respectively.
/// 0: F_OK, 1: X_OK, 2: W_OK, 4: R_OK
pub fn syscall_faccessat(dir_fd: usize, path: *const u8, mode: usize) -> SyscallResult {
    // todo: 有问题，实际上需要考虑当前进程对应的用户UID和文件拥有者之间的关系
    // 现在一律当作root用户处理
    let file_path = deal_with_path(dir_fd, Some(path), false).unwrap();
    axfs::api::metadata(file_path.path())
        .map(|metadata| {
            if mode == 0 {
                //F_OK
                // 文件存在返回0，不存在返回-1
                if axfs::api::path_exists(file_path.path()) {
                    Ok(0)
                } else {
                    Err(SyscallError::ENOENT)
                }
            } else {
                // 逐位对比
                let mut ret = true;
                if mode & 1 != 0 {
                    // X_OK
                    ret &= metadata.permissions().contains(Permissions::OWNER_EXEC)
                }
                if mode & 2 != 0 {
                    // W_OK
                    ret &= metadata.permissions().contains(Permissions::OWNER_WRITE)
                }
                if mode & 4 != 0 {
                    // R_OK
                    ret &= metadata.permissions().contains(Permissions::OWNER_READ)
                }
                Ok(ret as isize - 1)
            }
        })
        .unwrap_or_else(|_| Err(SyscallError::ENOENT))
}

/// 88
/// 用于修改文件或目录的时间戳(timestamp)
/// 如果 fir_fd < 0，它和 path 共同决定要找的文件；
/// 如果 fir_fd >=0，它就是文件对应的 fd
pub fn syscall_utimensat(
    dir_fd: usize,
    path: *const u8,
    times: *const TimeSecs,
    _flags: usize,
) -> SyscallResult {
    let process = current_process();
    // info!("dir_fd: {}, path: {}", dir_fd as usize, path as usize);
    if dir_fd != AT_FDCWD && (dir_fd as isize) < 0 {
        return Err(SyscallError::EBADF); // 错误的文件描述符
    }

    if dir_fd == AT_FDCWD
        && process
            .manual_alloc_for_lazy((path as usize).into())
            .is_err()
    {
        return Err(SyscallError::EFAULT); // 地址不合法
    }
    // 需要设置的时间
    let (new_atime, new_mtime) = if times.is_null() {
        (TimeSecs::now(), TimeSecs::now())
    } else {
        if process.manual_alloc_type_for_lazy(times).is_err() {
            return Err(SyscallError::EFAULT);
        }
        unsafe { (*times, *(times.add(1))) } //  注意传入的TimeVal中 sec和nsec都是usize, 但TimeValue中nsec是u32
    };
    // 感觉以下仿照maturin的实现不太合理，并没有真的把时间写给文件，只是写给了一个新建的临时的fd
    if (dir_fd as isize) > 0 {
        // let file = process_inner.fd_manager.fd_table[dir_fd].clone();
        // if !file.unwrap().lock().set_time(new_atime, new_mtime) {
        //     error!("Set time failed: unknown reason.");
        //     return ErrorNo::EPERM as isize;
        // }
        let fd_table = process.fd_manager.fd_table.lock();
        if dir_fd > fd_table.len() || fd_table[dir_fd].is_none() {
            return Err(SyscallError::EBADF);
        }
        if let Some(file) = fd_table[dir_fd].as_ref() {
            if let Some(fat_file) = file.as_any().downcast_ref::<FileDesc>() {
                // if !fat_file.set_time(new_atime, new_mtime) {
                //     error!("Set time failed: unknown reason.");
                //     return ErrorNo::EPERM as isize;
                // }
                fat_file.stat.lock().atime.set_as_utime(&new_atime);
                fat_file.stat.lock().mtime.set_as_utime(&new_mtime);
            } else {
                return Err(SyscallError::EPERM);
            }
        }
        Ok(0)
    } else {
        let file_path = deal_with_path(dir_fd, Some(path), false).unwrap();
        if !axfs::api::path_exists(file_path.path()) {
            error!("Set time failed: file {} doesn't exist!", file_path.path());
            if !axfs::api::path_exists(file_path.dir().unwrap()) {
                return Err(SyscallError::ENOTDIR);
            } else {
                return Err(SyscallError::ENOENT);
            }
        }
        let file = new_fd(file_path.path().to_string(), 0.into()).unwrap();
        file.stat.lock().atime.set_as_utime(&new_atime);
        file.stat.lock().mtime.set_as_utime(&new_mtime);
        Ok(0)
    }
}
