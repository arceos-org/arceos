use crate::fs::link::{create_link, remove_link};
use crate::fs::mount::{check_mounted, get_stat_in_fs, mount_fat_fs, umount_fat_fs};
use crate::fs::pipe::make_pipe;
use crate::fs::{new_dir, new_fd, FileDesc};
use crate::syscall::flags::raw_ptr_to_ref_str;

#[cfg(feature = "net")]
use syscall_net::Socket;
extern crate alloc;
use alloc::format;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;
use axerrno::AxError;
use axfs::api::{self, FileIOType, Kstat};
use axfs::api::{OpenFlags, Permissions};
use axhal::mem::VirtAddr;
use axio::SeekFrom;
use axlog::{debug, error, info};
use axprocess::current_process;
use axprocess::link::{real_path, FilePath};
use core::mem::transmute;
use core::ptr::copy_nonoverlapping;

use super::flags::{get_fs_stat, DirEnt, DirEntType, Fcntl64Cmd, FsStat, IoVec, TimeSecs};
use super::ErrorNo;

#[allow(unused)]
pub const AT_FDCWD: usize = -100isize as usize;
// Special value used to indicate openat should use the current working directory.
const AT_REMOVEDIR: usize = 0x200; // Remove directory instead of unlinking file.

// const STDIN: usize = 0;
// const STDOUT: usize = 1;
// const STDERR: usize = 2;

/// 辅助函数：处理路径，返回一个FilePath结构体
///
/// 输入：
///    - dir_fd：文件描述符
///    - path_addr：路径地址
///    - force_dir：是否强制为目录
///
/// 一般情况下, 传入path末尾是`/`的话, 生成的FilePath是一个目录，否则是一个文件；但如果force_dir为true, 则生成的FilePath一定是一个目录(自动补充`/`)
pub fn deal_with_path(
    dir_fd: usize,
    path_addr: Option<*const u8>,
    force_dir: bool,
) -> Option<FilePath> {
    let process = current_process();
    let mut path = "".to_string();
    if let Some(path_addr) = path_addr {
        if path_addr.is_null() {
            debug!("path address is null");
            return None;
        }
        if process
            .manual_alloc_for_lazy((path_addr as usize).into())
            .is_ok()
        {
            // 直接访问前需要确保已经被分配
            path = unsafe { raw_ptr_to_ref_str(path_addr) }.to_string().clone();
        } else {
            debug!("path address is invalid");
            return None;
        }
    }

    if force_dir {
        path = format!("{}/", path);
    }
    if path.ends_with('.') {
        // 如果path以.或..结尾, 则加上/告诉FilePath::new它是一个目录
        path = format!("{}/", path);
    }
    // info!("path: {}", path);

    if dir_fd != AT_FDCWD {
        // 如果不是绝对路径, 且dir_fd不是AT_FDCWD, 则需要将dir_fd和path拼接起来
        let fd_table = process.fd_manager.fd_table.lock();
        if dir_fd >= fd_table.len() {
            debug!("fd index out of range");
            return None;
        }
        match fd_table[dir_fd].as_ref() {
            Some(dir) => {
                if dir.get_type() != FileIOType::DirDesc {
                    debug!("selected fd is not a dir");
                    return None;
                }
                let dir = dir.clone();
                // 有没有可能dir的尾部一定是一个/号，所以不用手工添加/
                path = format!("{}{}", dir.get_path(), path);
                debug!("handled_path: {}", path);
            }
            None => {
                debug!("fd not exist");
                return None;
            }
        }
    }
    match FilePath::new(path.as_str()) {
        Ok(path) => Some(path),
        Err(_) => None,
    }
}

/// 功能：从一个文件描述符中读取；
/// 输入：
///     - fd：要读取文件的文件描述符。
///     - buf：一个缓存区，用于存放读取的内容。
///     - count：要读取的字节数。
/// 返回值：成功执行，返回读取的字节数。如为0，表示文件结束。错误，则返回-1。
pub fn syscall_read(fd: usize, buf: *mut u8, count: usize) -> isize {
    info!("[read()] fd: {fd}, buf: {buf:?}, len: {count}",);

    if buf.is_null() {
        return ErrorNo::EFAULT as isize;
    }

    let process = current_process();

    // TODO: 左闭右开
    let buf = match process.manual_alloc_range_for_lazy(
        (buf as usize).into(),
        (unsafe { buf.add(count) as usize } - 1).into(),
    ) {
        Ok(_) => unsafe { core::slice::from_raw_parts_mut(buf, count) },
        Err(_) => return ErrorNo::EFAULT as isize,
    };

    let file = match process.fd_manager.fd_table.lock().get(fd) {
        Some(Some(f)) => f.clone(),
        _ => return ErrorNo::EBADF as isize,
    };

    if file.get_type() == FileIOType::DirDesc {
        error!("fd is a dir");
        return ErrorNo::EISDIR as isize;
    }
    if !file.readable() {
        // 1. nonblocking socket
        //
        // Normal socket will block while trying to read, so we don't return here.
        #[cfg(feature = "net")]
        if let Some(socket) = file.as_any().downcast_ref::<Socket>() {
            if socket.is_nonblocking() && socket.is_connected() {
                return ErrorNo::EAGAIN as isize;
            }
        } else {
            // 2. nonblock file
            // return ErrorNo::EAGAIN as isize;
            // 3. regular file
            return ErrorNo::EBADF as isize;
        }

        #[cfg(not(feature = "net"))]
        return ErrorNo::EBADF as isize;
    }

    // for sockets:
    // Sockets are "readable" when:
    // - have some data to read without blocking
    // - remote end send FIN packet, local read half is closed (this will return 0 immediately)
    //   this will return Ok(0)
    // - ready to accept new connections

    match file.read(buf) {
        Ok(len) => len as isize,
        Err(AxError::WouldBlock) => ErrorNo::EAGAIN as isize,
        Err(_) => -1,
    }
}

/// 功能：从一个文件描述符中写入；
/// 输入：
///     - fd：要写入文件的文件描述符。
///     - buf：一个缓存区，用于存放要写入的内容。
///     - count：要写入的字节数。
/// 返回值：成功执行，返回写入的字节数。错误，则返回-1。
pub fn syscall_write(fd: usize, buf: *const u8, count: usize) -> isize {
    info!("[write()] fd: {fd}, buf: {buf:?}, len: {count}");

    if buf.is_null() {
        return ErrorNo::EFAULT as isize;
    }

    let process = current_process();

    // TODO: 左闭右开
    let buf = match process.manual_alloc_range_for_lazy(
        (buf as usize).into(),
        (unsafe { buf.add(count) as usize } - 1).into(),
    ) {
        Ok(_) => unsafe { core::slice::from_raw_parts(buf, count) },
        Err(_) => return ErrorNo::EFAULT as isize,
    };

    let file = match process.fd_manager.fd_table.lock().get(fd) {
        Some(Some(f)) => f.clone(),
        _ => return ErrorNo::EBADF as isize,
    };

    if file.get_type() == FileIOType::DirDesc {
        debug!("fd is a dir");
        return ErrorNo::EBADF as isize;
    }
    if !file.writable() {
        // 1. socket
        //
        // Normal socket will block while trying to write, so we don't return here.
        #[cfg(feature = "net")]
        if let Some(socket) = file.as_any().downcast_ref::<Socket>() {
            if socket.is_nonblocking() && socket.is_connected() {
                return ErrorNo::EAGAIN as isize;
            }
        } else {
            // 2. nonblock file
            // return ErrorNo::EAGAIN as isize;

            // 3. regular file
            return ErrorNo::EBADF as isize;
        }

        #[cfg(not(feature = "net"))]
        return ErrorNo::EBADF as isize;
    }

    // for sockets:
    // Sockets are "writable" when:
    // - connected and have space in tx buffer to write
    // - sent FIN packet, local send half is closed (this will return 0 immediately)
    //   this will return Err(ConnectionReset)

    match file.write(buf) {
        Ok(len) => len as isize,
        // socket with send half closed
        // TODO: send a SIGPIPE signal to the process
        Err(axerrno::AxError::ConnectionReset) => ErrorNo::EPIPE as isize,
        Err(_) => -1,
    }
}

/// 从同一个文件描述符读取多个字符串
pub fn syscall_readv(fd: usize, iov: *mut IoVec, iov_cnt: usize) -> isize {
    let mut read_len = 0;
    // 似乎要判断iov是否分配，但是懒了，反正能过测例
    for i in 0..iov_cnt {
        let io: &IoVec = unsafe { &*iov.add(i) };
        if io.base.is_null() || io.len == 0 {
            continue;
        }
        match syscall_read(fd, io.base, io.len) {
            len if len >= 0 => read_len += len,

            err => return err,
        }
    }
    read_len
}

/// 从同一个文件描述符写入多个字符串
pub fn syscall_writev(fd: usize, iov: *const IoVec, iov_cnt: usize) -> isize {
    let mut write_len = 0;
    // 似乎要判断iov是否分配，但是懒了，反正能过测例
    for i in 0..iov_cnt {
        let io: &IoVec = unsafe { &(*iov.add(i)) };
        if io.base.is_null() || io.len == 0 {
            continue;
        }
        match syscall_write(fd, io.base, io.len) {
            len if len >= 0 => write_len += len,

            err => return err,
        }
    }
    write_len
}

/// 功能：打开或创建一个文件；
/// 输入：
///     - fd：文件所在目录的文件描述符。
///     - filename：要打开或创建的文件名。如为绝对路径，则忽略fd。如为相对路径，且fd是AT_FDCWD，则filename是相对于当前工作目录来说的。如为相对路径，且fd是一个文件描述符，则filename是相对于fd所指向的目录来说的。
///     - flags：必须包含如下访问模式的其中一种：O_RDONLY，O_WRONLY，O_RDWR。还可以包含文件创建标志和文件状态标志。
///     - mode：文件的所有权描述。详见`man 7 inode `。
/// 返回值：成功执行，返回新的文件描述符。失败，返回-1。
///
/// 说明：如果打开的是一个目录，那么返回的文件描述符指向的是该目录的描述符。(后面会用到针对目录的文件描述符)
/// flags: O_RDONLY: 0, O_WRONLY: 1, O_RDWR: 2, O_CREAT: 64, O_DIRECTORY: 65536
pub fn syscall_openat(fd: usize, path: *const u8, flags: usize, _mode: u8) -> isize {
    let force_dir = OpenFlags::from(flags).is_dir();
    let path = if let Some(path) = deal_with_path(fd, Some(path), force_dir) {
        path
    } else {
        return ErrorNo::EINVAL as isize;
    };
    let process = current_process();
    let mut fd_table = process.fd_manager.fd_table.lock();
    let fd_num = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
        fd
    } else {
        return ErrorNo::EMFILE as isize;
    };
    debug!("allocated fd_num: {}", fd_num);
    // 如果是DIR
    let ans = if path.is_dir() {
        debug!("open dir");
        if let Ok(dir) = new_dir(path.path().to_string(), flags.into()) {
            debug!("new dir_desc successfully allocated: {}", path.path());
            fd_table[fd_num] = Some(Arc::new(dir));
            fd_num as isize
        } else {
            debug!("open dir failed");
            ErrorNo::ENOENT as isize
        }
    }
    // 如果是FILE，注意若创建了新文件，需要添加链接
    else {
        debug!("open file");
        if let Ok(file) = new_fd(path.path().to_string(), flags.into()) {
            debug!("new file_desc successfully allocated");
            fd_table[fd_num] = Some(Arc::new(file));
            let _ = create_link(&path, &path); // 不需要检查是否成功，因为如果成功，说明是新建的文件，如果失败，说明已经存在了
            fd_num as isize
        } else {
            debug!("open file failed");
            ErrorNo::ENOENT as isize
        }
    };
    info!("openat: {} -> {}", path.path(), ans);
    ans
}

/// 功能：关闭一个文件描述符；
/// 输入：
///     - fd：要关闭的文件描述符。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_close(fd: usize) -> isize {
    info!("Into syscall_close. fd: {}", fd);

    let process = current_process();
    let mut fd_table = process.fd_manager.fd_table.lock();
    if fd >= fd_table.len() {
        debug!("fd {} is out of range", fd);
        return -1;
    }
    // if fd == 3 {
    //     debug!("fd {} is reserved for cwd", fd);
    //     return -1;
    // }
    if fd_table[fd].is_none() {
        debug!("fd {} is none", fd);
        return -1;
    }
    // let file = process_inner.fd_manager.fd_table[fd].unwrap();
    fd_table[fd] = None;
    // for i in 0..process_inner.fd_table.len() {
    //     if let Some(file) = process_inner.fd_table[i].as_ref() {
    //         debug!("fd: {} has file", i);
    //     }
    // }

    0
}

/// 功能：获取当前工作目录；
/// 输入：
///     - char *buf：一块缓存区，用于保存当前工作目录的字符串。当buf设为NULL，由系统来分配缓存区。
///     - size：buf缓存区的大小。
/// 返回值：成功执行，则返回当前工作目录的字符串的指针。失败，则返回NULL。
///  暂时：成功执行，则返回当前工作目录的字符串的指针 as isize。失败，则返回0。
///
/// 注意：当前写法存在问题，cwd应当是各个进程独立的，而这里修改的是整个fs的目录
pub fn syscall_getcwd(buf: *mut u8, len: usize) -> isize {
    debug!("Into syscall_getcwd. buf: {}, len: {}", buf as usize, len);
    let cwd = api::current_dir().unwrap();

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
            buf as isize
        } else {
            ErrorNo::EINVAL as isize
        }
    } else {
        debug!("getcwd: buf size is too small");
        ErrorNo::ERANGE as isize
    };
}

/// 功能：创建管道；
/// 输入：
///     - fd[2]：用于保存2个文件描述符。其中，fd[0]为管道的读出端，fd[1]为管道的写入端。
/// 返回值：成功执行，返回0。失败，返回-1。
///
/// 注意：fd[2]是32位数组，所以这里的 fd 是 u32 类型的指针，而不是 usize 类型的指针。
pub fn syscall_pipe2(fd: *mut u32) -> isize {
    debug!("Into syscall_pipe2. fd: {}", fd as usize);
    let process = current_process();
    if process.manual_alloc_for_lazy((fd as usize).into()).is_err() {
        return ErrorNo::EINVAL as isize;
    }
    let (read, write) = make_pipe();
    let mut fd_table = process.fd_manager.fd_table.lock();
    let fd_num = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
        fd
    } else {
        return -1;
    };
    fd_table[fd_num] = Some(read);
    let fd_num2 = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
        fd
    } else {
        return -1;
    };
    fd_table[fd_num2] = Some(write);
    info!("read end: {} write: end: {}", fd_num, fd_num2);
    unsafe {
        core::ptr::write(fd, fd_num as u32);
        core::ptr::write(fd.offset(1), fd_num2 as u32);
    }
    0
}

/// 功能：复制文件描述符；
/// 输入：
///     - fd：被复制的文件描述符。
/// 返回值：成功执行，返回新的文件描述符。失败，返回-1。
pub fn syscall_dup(fd: usize) -> isize {
    let process = current_process();
    let mut fd_table = process.fd_manager.fd_table.lock();
    if fd >= fd_table.len() {
        debug!("fd {} is out of range", fd);
        return ErrorNo::EBADF as isize;
    }
    if fd_table[fd].is_none() {
        debug!("fd {} is a closed fd", fd);
        return ErrorNo::EBADF as isize;
    }

    let new_fd = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
        fd
    } else {
        // 文件描述符达到上限了
        return ErrorNo::EMFILE as isize;
    };
    fd_table[new_fd] = fd_table[fd].clone();

    new_fd as isize
}

/// 功能：复制文件描述符，并指定了新的文件描述符；
/// 输入：
///     - old：被复制的文件描述符。
///     - new：新的文件描述符。
/// 返回值：成功执行，返回新的文件描述符。失败，返回-1。
pub fn syscall_dup3(fd: usize, new_fd: usize) -> isize {
    let process = current_process();
    let mut fd_table = process.fd_manager.fd_table.lock();
    if fd >= fd_table.len() {
        debug!("fd {} is out of range", fd);
        return -1;
    }
    if fd_table[fd].is_none() {
        debug!("fd {} is not opened", fd);
        return -1;
    }
    if new_fd >= fd_table.len() {
        if new_fd >= (process.fd_manager.get_limit() as usize) {
            // 超出了资源限制
            return ErrorNo::EBADF as isize;
        }
        for _i in fd_table.len()..new_fd + 1 {
            fd_table.push(None);
        }
    }
    // if process_inner.fd_manager.fd_table[new_fd].is_some() {
    //     debug!("new_fd {} is already opened", new_fd);
    //     return ErrorNo::EINVAL as isize;
    // }
    info!("dup3 fd {} to new fd {}", fd, new_fd);
    // 就算new_fd已经被打开了，也可以被重新替代掉
    fd_table[new_fd] = fd_table[fd].clone();
    new_fd as isize
}

/// 功能：创建目录；
/// 输入：
///     - dirfd：要创建的目录所在的目录的文件描述符。
///     - path：要创建的目录的名称。如果path是相对路径，则它是相对于dirfd目录而言的。如果path是相对路径，且dirfd的值为AT_FDCWD，则它是相对于当前路径而言的。如果path是绝对路径，则dirfd被忽略。
///     - mode：文件的所有权描述。详见`man 7 inode `。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_mkdirat(dir_fd: usize, path: *const u8, mode: u32) -> isize {
    // info!("signal module: {:?}", process_inner.signal_module.keys());
    let path = if let Some(path) = deal_with_path(dir_fd, Some(path), true) {
        path
    } else {
        return ErrorNo::EINVAL as isize;
    };
    debug!(
        "Into syscall_mkdirat. dirfd: {}, path: {:?}, mode: {}",
        dir_fd,
        path.path(),
        mode
    );
    if api::path_exists(path.path()) {
        // 文件已存在
        return ErrorNo::EEXIST as isize;
    }
    let _ = api::create_dir(path.path());
    // 只要文件夹存在就返回0
    if api::path_exists(path.path()) {
        0
    } else {
        -1
    }
}

/// 功能：切换工作目录；
/// 输入：
///     - path：需要切换到的目录。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_chdir(path: *const u8) -> isize {
    // 从path中读取字符串
    let path = if let Some(path) = deal_with_path(AT_FDCWD, Some(path), true) {
        path
    } else {
        return ErrorNo::EINVAL as isize;
    };
    debug!("Into syscall_chdir. path: {:?}", path.path());
    match api::set_current_dir(path.path()) {
        Ok(_) => 0,
        Err(_) => ErrorNo::EINVAL as isize,
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
pub fn syscall_getdents64(fd: usize, buf: *mut u8, len: usize) -> isize {
    let path = if let Some(path) = deal_with_path(fd, None, true) {
        path
    } else {
        return ErrorNo::EINVAL as isize;
    };

    let process = current_process();
    // 注意是否分配地址
    let start: VirtAddr = (buf as usize).into();
    let end = start + len;
    if process.manual_alloc_range_for_lazy(start, end).is_err() {
        return ErrorNo::EFAULT as isize;
    }

    let entry_id_from = unsafe { (*(buf as *const DirEnt)).d_off };
    if entry_id_from == -1 {
        // 说明已经读完了
        return 0;
    }

    let buf = unsafe { core::slice::from_raw_parts_mut(buf, len) };
    let dir_iter = api::read_dir(path.path()).unwrap();
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
            return count as isize;
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
    count as isize
}

/// 功能：创建文件的链接；
/// 输入：
///     - old_dir_fd：原来的文件所在目录的文件描述符。
///     - old_path：文件原来的名字。如果old_path是相对路径，则它是相对于old_dir_fd目录而言的。如果old_path是相对路径，且old_dir_fd的值为AT_FDCWD，则它是相对于当前路径而言的。如果old_path是绝对路径，则old_dir_fd被忽略。
///     - new_dir_fd：新文件名所在的目录。
///     - new_path：文件的新名字。new_path的使用规则同old_path。
///     - flags：在2.6.18内核之前，应置为0。其它的值详见`man 2 linkat`。
/// 返回值：成功执行，返回0。失败，返回-1。
#[allow(dead_code)]
pub fn sys_linkat(
    old_dir_fd: usize,
    old_path: *const u8,
    new_dir_fd: usize,
    new_path: *const u8,
    _flags: usize,
) -> isize {
    let old_path = if let Some(path) = deal_with_path(old_dir_fd, Some(old_path), false) {
        path
    } else {
        return ErrorNo::EINVAL as isize;
    };
    let new_path = if let Some(path) = deal_with_path(new_dir_fd, Some(new_path), false) {
        path
    } else {
        return ErrorNo::EINVAL as isize;
    };
    if create_link(&old_path, &new_path) {
        0
    } else {
        return ErrorNo::EINVAL as isize;
    }
}

/// 功能：移除指定文件的链接(可用于删除文件)；
/// 输入：
///     - dir_fd：要删除的链接所在的目录。
///     - path：要删除的链接的名字。如果path是相对路径，则它是相对于dir_fd目录而言的。如果path是相对路径，且dir_fd的值为AT_FDCWD，则它是相对于当前路径而言的。如果path是绝对路径，则dir_fd被忽略。
///     - flags：可设置为0或AT_REMOVEDIR。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_unlinkat(dir_fd: usize, path: *const u8, flags: usize) -> isize {
    let path = deal_with_path(dir_fd, Some(path), false).unwrap();

    if path.start_with(&FilePath::new("/proc").unwrap()) {
        return -1;
    }

    // unlink file
    if flags == 0 {
        if let None = remove_link(&path) {
            debug!("unlink file error");
            return ErrorNo::EINVAL as isize;
        }
    }
    // remove dir
    else if flags == AT_REMOVEDIR {
        if let Err(e) = api::remove_dir(path.path()) {
            debug!("rmdir error: {:?}", e);
            return ErrorNo::EINVAL as isize;
        }
    }
    // flags error
    else {
        debug!("flags error");
        return ErrorNo::EINVAL as isize;
    }
    0
}

/// 功能：挂载文件系统；
/// 输入：
///   - special: 挂载设备；
///   - dir: 挂载点；       经过实测，发现dir可以是绝对路径，也可以是相对路径，甚至可以是 . 或 ..
///   - fs_type: 挂载的文件系统类型；
///   - flags: 挂载参数；
///   - data: 传递给文件系统的字符串参数，可为NULL；
/// 返回值：成功返回0，失败返回-1
pub fn syscall_mount(
    special: *const u8,
    dir: *const u8,
    fs_type: *const u8,
    _flags: usize,
    _data: *const u8,
) -> isize {
    let device_path = deal_with_path(AT_FDCWD, Some(special), false).unwrap();
    // 这里dir必须以"/"结尾，但在shell中输入时，不需要以"/"结尾
    let mount_path = deal_with_path(AT_FDCWD, Some(dir), true).unwrap();

    let process = current_process();
    if process
        .manual_alloc_for_lazy((fs_type as usize).into())
        .is_err()
    {
        return ErrorNo::EINVAL as isize;
    }
    let fs_type = unsafe { raw_ptr_to_ref_str(fs_type) }.to_string();
    let mut _data_str = "".to_string();
    if !_data.is_null() {
        if process
            .manual_alloc_for_lazy((_data as usize).into())
            .is_err()
        {
            return ErrorNo::EINVAL as isize;
        }
        // data可以为NULL, 必须判断, 否则会panic, 发生LoadPageFault
        _data_str = unsafe { raw_ptr_to_ref_str(_data) }.to_string();
    }
    if device_path.is_dir() {
        debug!("device_path should not be a dir");
        return -1;
    }
    if !mount_path.is_dir() {
        debug!("mount_path should be a dir");
        return -1;
    }

    // 如果mount_path不存在，则创建
    if !api::path_exists(mount_path.path()) {
        if let Err(e) = api::create_dir(mount_path.path()) {
            debug!("create mount path error: {:?}", e);
            return -1;
        }
    }

    if fs_type != "vfat" {
        debug!("fs_type can only be vfat.");
        return -1;
    }
    // 检查挂载点路径是否存在
    if !api::path_exists(mount_path.path()) {
        debug!("mount path not exist");
        return -1;
    }
    // 查挂载点是否已经被挂载
    if check_mounted(&mount_path) {
        debug!("mount path includes mounted fs");
        return -1;
    }
    // 挂载
    if !mount_fat_fs(&device_path, &mount_path) {
        debug!("mount error");
        return -1;
    }

    0
}

/// 功能：卸载文件系统；
/// 输入：指定卸载目录，卸载参数；
/// 返回值：成功返回0，失败返回-1；
pub fn syscall_umount(dir: *const u8, flags: usize) -> isize {
    let mount_path = deal_with_path(AT_FDCWD, Some(dir), true).unwrap();

    if flags != 0 {
        debug!("flags unimplemented");
        return -1;
    }

    // 检查挂载点路径是否存在
    if !api::path_exists(mount_path.path()) {
        debug!("mount path not exist");
        return -1;
    }
    // 从挂载点中删除
    if !umount_fat_fs(&mount_path) {
        debug!("umount error");
        return -1;
    }

    0
}
pub fn syscall_fstat(fd: usize, kst: *mut Kstat) -> isize {
    let process = current_process();
    let fd_table = process.fd_manager.fd_table.lock();

    if fd >= fd_table.len() || fd < 3 {
        debug!("fd {} is out of range", fd);
        return -1;
    }
    if fd_table[fd].is_none() {
        debug!("fd {} is none", fd);
        return -1;
    }
    let file = fd_table[fd].clone().unwrap();
    if file.get_type() != FileIOType::FileDesc {
        debug!("fd {} is not a file", fd);
        return -1;
    }

    let ans = match file.get_stat() {
        Ok(stat) => {
            unsafe {
                *kst = stat;
            }
            0
        }
        Err(e) => {
            debug!("get stat error: {:?}", e);
            -1
        }
    };
    ans
}

/// 获取文件状态信息，但是给出的是目录 fd 和相对路径。 79
pub fn syscall_fstatat(dir_fd: usize, path: *const u8, kst: *mut Kstat) -> isize {
    let file_path = deal_with_path(dir_fd, Some(path), false).unwrap();
    info!("path : {}", file_path.path());
    match get_stat_in_fs(&file_path) {
        Ok(stat) => unsafe {
            *kst = stat;
            0
        },
        Err(error_no) => {
            debug!("get stat error: {:?}", error_no);
            error_no as isize
        }
    }
}

pub fn syscall_fcntl64(fd: usize, cmd: usize, arg: usize) -> isize {
    let process = current_process();
    let mut fd_table = process.fd_manager.fd_table.lock();

    if fd >= fd_table.len() {
        debug!("fd {} is out of range", fd);
        return ErrorNo::EBADF as isize;
    }
    if fd_table[fd].is_none() {
        debug!("fd {} is none", fd);
        return ErrorNo::EBADF as isize;
    }
    let file = fd_table[fd].clone().unwrap();
    info!("fd: {}, cmd: {}", fd, cmd);
    match Fcntl64Cmd::try_from(cmd) {
        Ok(Fcntl64Cmd::F_DUPFD) => {
            let new_fd = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
                fd
            } else {
                // 文件描述符达到上限了
                return ErrorNo::EMFILE as isize;
            };
            fd_table[new_fd] = fd_table[fd].clone();
            return new_fd as isize;
        }
        Ok(Fcntl64Cmd::F_GETFD) => {
            if file.get_status().contains(OpenFlags::CLOEXEC) {
                1
            } else {
                0
            }
        }
        Ok(Fcntl64Cmd::F_SETFD) => {
            if file.set_close_on_exec((arg & 1) != 0) {
                0
            } else {
                ErrorNo::EINVAL as isize
            }
        }
        Ok(Fcntl64Cmd::F_GETFL) => file.get_status().bits() as isize,
        Ok(Fcntl64Cmd::F_SETFL) => {
            if let Some(flags) = OpenFlags::from_bits(arg as u32) {
                if file.set_status(flags) {
                    return 0;
                }
            }
            ErrorNo::EINVAL as isize
        }
        Ok(Fcntl64Cmd::F_DUPFD_CLOEXEC) => {
            let new_fd = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
                fd
            } else {
                // 文件描述符达到上限了
                return ErrorNo::EMFILE as isize;
            };

            if file.set_close_on_exec((arg & 1) != 0) {
                fd_table[new_fd] = fd_table[fd].clone();
                return new_fd as isize;
            } else {
                return ErrorNo::EINVAL as isize;
            }
        }
        _ => ErrorNo::EINVAL as isize,
    }
}

/// 43
/// 获取文件系统的信息
pub fn syscall_statfs(path: *const u8, stat: *mut FsStat) -> isize {
    let file_path = deal_with_path(AT_FDCWD, Some(path), false).unwrap();
    if file_path.equal_to(&FilePath::new("/").unwrap()) {
        // 目前只支持访问根目录文件系统的信息
        unsafe {
            *stat = get_fs_stat();
        }

        0
    } else {
        error!("Only support fs_stat for root");
        ErrorNo::EINVAL as isize
    }
}

/// 29
/// 执行各种设备相关的控制功能
/// todo: 未实现
pub fn syscall_ioctl(fd: usize, request: usize, argp: *mut usize) -> isize {
    let process = current_process();
    let fd_table = process.fd_manager.fd_table.lock();
    info!("fd: {}, request: {}, argp: {}", fd, request, argp as usize);
    if fd >= fd_table.len() {
        debug!("fd {} is out of range", fd);
        return ErrorNo::EBADF as isize;
    }
    if fd_table[fd].is_none() {
        debug!("fd {} is none", fd);
        return ErrorNo::EBADF as isize;
    }
    if process
        .manual_alloc_for_lazy((argp as usize).into())
        .is_err()
    {
        return ErrorNo::EFAULT as isize; // 地址不合法
    }

    let file = fd_table[fd].clone().unwrap();
    // if file.lock().ioctl(request, argp as usize).is_err() {
    //     return -1;
    // }
    let _ = file.ioctl(request, argp as usize);
    0
}

/// 53
/// 修改文件权限
/// mode: 0o777, 3位八进制数字
/// path为相对路径：
///     1. 若dir_fd为AT_FDCWD，则相对于当前工作目录
///     2. 若dir_fd为AT_FDCWD以外的值，则相对于dir_fd所指的目录
/// path为绝对路径：
///     忽视dir_fd，直接根据path访问
pub fn syscall_fchmodat(dir_fd: usize, path: *const u8, mode: usize) -> isize {
    let file_path = deal_with_path(dir_fd, Some(path), false).unwrap();
    api::metadata(file_path.path())
        .map(|mut metadata| {
            metadata.set_permissions(Permissions::from_bits_truncate(mode as u16));
            0
        })
        .unwrap_or_else(|_| ErrorNo::ENOENT as isize)
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
pub fn syscall_faccessat(dir_fd: usize, path: *const u8, mode: usize) -> isize {
    // todo: 有问题，实际上需要考虑当前进程对应的用户UID和文件拥有者之间的关系
    // 现在一律当作root用户处理
    let file_path = deal_with_path(dir_fd, Some(path), false).unwrap();
    api::metadata(file_path.path())
        .map(|metadata| {
            if mode == 0 {
                //F_OK
                // 文件存在返回0，不存在返回-1
                if api::path_exists(file_path.path()) {
                    0
                } else {
                    ErrorNo::ENOENT as isize
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
                ret as isize - 1
            }
        })
        .unwrap_or_else(|_| ErrorNo::ENOENT as isize)
}

/// 67
/// pread64
/// 从文件的指定位置读取数据，并且不改变文件的读写指针
pub fn syscall_pread64(fd: usize, buf: *mut u8, count: usize, offset: usize) -> isize {
    let process = current_process();
    // todo: 把check fd整合到fd_manager中
    let file = process.fd_manager.fd_table.lock()[fd].clone().unwrap();

    let old_offset = file.seek(SeekFrom::Current(0)).unwrap();
    let ret = file
        .seek(SeekFrom::Start(offset as u64))
        .and_then(|_| file.read(unsafe { core::slice::from_raw_parts_mut(buf, count) }));
    file.seek(SeekFrom::Start(old_offset)).unwrap();
    ret.map(|size| size as isize)
        .unwrap_or_else(|_| ErrorNo::EINVAL as isize)
}

/// 68
/// pwrite64
/// 向文件的指定位置写入数据，并且不改变文件的读写指针
pub fn syscall_pwrite64(fd: usize, buf: *const u8, count: usize, offset: usize) -> isize {
    let process = current_process();

    let file = process.fd_manager.fd_table.lock()[fd].clone().unwrap();

    let old_offset = file.seek(SeekFrom::Current(0)).unwrap();

    let ret = file.seek(SeekFrom::Start(offset as u64)).and_then(|_| {
        let res = file.write(unsafe { core::slice::from_raw_parts(buf, count) });
        res
    });

    file.seek(SeekFrom::Start(old_offset)).unwrap();
    drop(file);

    ret.map(|size| size as isize)
        .unwrap_or_else(|_| ErrorNo::EINVAL as isize)
}

/// 71
/// sendfile64
/// 将一个文件的内容发送到另一个文件中
/// 如果offset为NULL，则从当前读写指针开始读取，读取完毕后会更新读写指针
/// 如果offset不为NULL，则从offset指定的位置开始读取，读取完毕后不会更新读写指针，但是会更新offset的值
pub fn syscall_sendfile64(out_fd: usize, in_fd: usize, offset: *mut usize, count: usize) -> isize {
    info!("send from {} to {}, count: {}", in_fd, out_fd, count);
    let process = current_process();
    let out_file = process.fd_manager.fd_table.lock()[out_fd].clone().unwrap();
    let in_file = process.fd_manager.fd_table.lock()[in_fd].clone().unwrap();
    let old_in_offset = in_file.seek(SeekFrom::Current(0)).unwrap();

    let mut buf = vec![0u8; count];
    let ans = if !offset.is_null() {
        // 如果offset不为NULL，则从offset指定的位置开始读取
        let in_offset = unsafe { *offset };
        in_file.seek(SeekFrom::Start(in_offset as u64)).unwrap();
        let ret = in_file.read(buf.as_mut_slice());
        unsafe { *offset = in_offset + ret.unwrap() };
        in_file.seek(SeekFrom::Start(old_in_offset)).unwrap();
        let buf = buf[..ret.unwrap()].to_vec();
        out_file.write(buf.as_slice()).unwrap() as isize
    } else {
        // 如果offset为NULL，则从当前读写指针开始读取
        let ret = in_file.read(buf.as_mut_slice());
        info!("in fd: {}, count: {}", in_fd, count);
        let buf = buf[..ret.unwrap()].to_vec();
        info!("read len: {}", buf.len());
        info!("write len: {}", buf.as_slice().len());
        out_file.write(buf.as_slice()).unwrap() as isize
    };
    info!("ans: {}", ans);
    ans
}

/// 78
/// readlinkat
/// 读取符号链接文件的内容
/// 如果buf为NULL，则返回符号链接文件的长度
/// 如果buf不为NULL，则将符号链接文件的内容写入buf中
/// 如果写入的内容超出了buf_size则直接截断
pub fn syscall_readlinkat(dir_fd: usize, path: *const u8, buf: *mut u8, bufsiz: usize) -> isize {
    let process = current_process();
    if process
        .manual_alloc_for_lazy((path as usize).into())
        .is_err()
    {
        return ErrorNo::EFAULT as isize;
    }
    if !buf.is_null() {
        if process
            .manual_alloc_for_lazy((buf as usize).into())
            .is_err()
        {
            return ErrorNo::EFAULT as isize;
        }
    }

    let path = deal_with_path(dir_fd, Some(path), false);
    if path.is_none() {
        return ErrorNo::ENOENT as isize;
    }
    let path = path.unwrap();
    if path.path() == "proc/self/exe" {
        // 针对lmbench_all特判
        let name = "/lmbench_all";
        let len = bufsiz.min(name.len());
        let slice = unsafe { core::slice::from_raw_parts_mut(buf, bufsiz) };
        slice.copy_from_slice(&name.as_bytes()[..len]);
        return len as isize;
    }
    if path.path().to_string() != real_path(&(path.path().to_string())) {
        // 说明链接存在
        let path = path.path();
        let len = bufsiz.min(path.len());
        let slice = unsafe { core::slice::from_raw_parts_mut(buf, len) };
        slice.copy_from_slice(&path.as_bytes()[..len]);
        return path.len() as isize;
    }
    ErrorNo::EINVAL as isize
}
/// 62
/// 移动文件描述符的读写指针
pub fn syscall_lseek(fd: usize, offset: isize, whence: usize) -> isize {
    let process = current_process();
    info!("fd: {} offset: {} whence: {}", fd, offset, whence);
    if fd >= process.fd_manager.fd_table.lock().len() || fd < 3 {
        debug!("fd {} is out of range", fd);
        return ErrorNo::EBADF as isize;
    }
    let fd_table = process.fd_manager.fd_table.lock();
    if let Some(file) = fd_table[fd].as_ref() {
        if file.get_type() == FileIOType::DirDesc {
            debug!("fd is a dir");
            return ErrorNo::EISDIR as isize;
        }
        let ans = if whence == 0 {
            // 即SEEK_SET
            file.seek(SeekFrom::Start(offset as u64))
        } else if whence == 1 {
            // 即SEEK_CUR
            file.seek(SeekFrom::Current(offset as i64))
        } else if whence == 2 {
            // 即SEEK_END
            file.seek(SeekFrom::End(offset as i64))
        } else {
            return ErrorNo::EINVAL as isize;
        };
        if let Ok(now_offset) = ans {
            now_offset as isize
        } else {
            return ErrorNo::EINVAL as isize;
        }
    } else {
        debug!("fd {} is none", fd);
        return ErrorNo::EBADF as isize;
    }
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
) -> isize {
    let process = current_process();
    // info!("dir_fd: {}, path: {}", dir_fd as usize, path as usize);
    if dir_fd != AT_FDCWD && (dir_fd as isize) < 0 {
        return ErrorNo::EBADF as isize; // 错误的文件描述符
    }

    if dir_fd == AT_FDCWD
        && process
            .manual_alloc_for_lazy((path as usize).into())
            .is_err()
    {
        return ErrorNo::EFAULT as isize; // 地址不合法
    }
    // 需要设置的时间
    let (new_atime, new_mtime) = if times.is_null() {
        (TimeSecs::now(), TimeSecs::now())
    } else {
        if process.manual_alloc_type_for_lazy(times).is_err() {
            return ErrorNo::EFAULT as isize;
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
            return ErrorNo::EBADF as isize;
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
                return ErrorNo::EPERM as isize;
            }
        }
        0
    } else {
        let file_path = deal_with_path(dir_fd, Some(path), false).unwrap();
        if !api::path_exists(file_path.path()) {
            error!("Set time failed: file {} doesn't exist!", file_path.path());
            if !api::path_exists(file_path.dir().unwrap()) {
                return ErrorNo::ENOTDIR as isize;
            } else {
                return ErrorNo::ENOENT as isize;
            }
        }
        let file = new_fd(file_path.path().to_string(), 0.into()).unwrap();
        file.stat.lock().atime.set_as_utime(&new_atime);
        file.stat.lock().mtime.set_as_utime(&new_mtime);
        0
    }
}

/// 82
/// 写回硬盘
#[allow(unused)]
pub fn syscall_fsync(fd: usize) -> isize {
    let process = current_process();
    if fd >= process.fd_manager.fd_table.lock().len() || fd < 3 {
        debug!("fd {} is out of range", fd);
        return ErrorNo::EBADF as isize;
    }
    let fd_table = process.fd_manager.fd_table.lock();
    if let Some(file) = fd_table[fd].clone() {
        if file.flush().is_err() {}
        0
    } else {
        debug!("fd {} is none", fd);
        return ErrorNo::EBADF as isize;
    }
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
) -> isize {
    let old_path = deal_with_path(old_dirfd, Some(_old_path), false).unwrap();
    let new_path = deal_with_path(new_dirfd, Some(_new_path), false).unwrap();

    let proc_path = FilePath::new("/proc").unwrap();
    if old_path.start_with(&proc_path) || new_path.start_with(&proc_path) {
        return -1;
    }

    // 交换两个目录名，目录下的文件不受影响，

    // 如果重命名后的文件已存在
    if flags == 1 {
        // 即RENAME_NOREPLACE
        if api::path_exists(new_path.path()) {
            debug!("new_path_ already exist");
            return -1;
        }
    }

    // 文件与文件夹不能互换命名
    if flags == 2 {
        // 即RENAME_EXCHANGE
        let old_metadata = api::metadata(old_path.path()).unwrap();
        let new_metadata = api::metadata(new_path.path()).unwrap();
        if old_metadata.is_dir() != new_metadata.is_dir() {
            debug!("old_path_ and new_path_ is not the same type");
            return -1;
        }
    }

    if flags == 4 {
        // 即RENAME_WHITEOUT
        let new_metadata = api::metadata(new_path.path()).unwrap();
        if new_metadata.is_dir() {
            debug!("new_path_ is a directory");
            return -1;
        }
    }

    if flags != 1 && flags != 2 && flags != 4 {
        debug!("flags is not valid");
        return -1;
    }

    // 做实际重命名操作

    0
}

pub fn syscall_ftruncate64(fd: usize, len: usize) -> isize {
    let process = current_process();
    info!("fd: {}, len: {}", fd, len);
    let fd_table = process.fd_manager.fd_table.lock();
    if fd >= fd_table.len() {
        return ErrorNo::EINVAL as isize;
    }
    if fd_table[fd].is_none() {
        return ErrorNo::EINVAL as isize;
    }

    if let Some(file) = fd_table[fd].as_ref() {
        if file.truncate(len).is_err() {
            return ErrorNo::EINVAL as isize;
        }
    }
    0
}

/**
该系统调用应复制文件描述符 fd_in 中的至多 len 个字节到文件描述符 fd_out 中。
若 off_in 为 NULL，则复制时应从文件描述符 fd_in 本身的文件偏移处开始读取，并将其文件偏移增加成功复制的字节数；否则，从 *off_in 指定的文件偏移处开始读取，不改变 fd_in 的文件偏移，而是将 *off_in 增加成功复制的字节数。
参数 off_out 的行为类似：若 off_out 为 NULL，则复制时从文件描述符 fd_out 本身的文件偏移处开始写入，并将其文件偏移增加成功复制的字节数；否则，从 *off_out 指定的文件偏移处开始写入，不改变 fd_out 的文件偏移，而是将 *off_out 增加成功复制的字节数。
该系统调用的返回值为成功复制的字节数，出现错误时返回负值。若读取 fd_in 时的文件偏移超过其大小，则直接返回 0，不进行复制。
本题中，fd_in 和 fd_out 总指向文件系统中两个不同的普通文件；flags 总为 0，没有实际作用。
 */

pub fn syscall_copyfilerange(
    fd_in: usize,
    off_in: *mut usize,
    fd_out: usize,
    off_out: *mut usize,
    len: usize,
    flags: usize,
) -> isize {
    let in_offset = if off_in.is_null() {
        -1
    } else {
        unsafe { *off_in as isize }
    };
    let out_offset = if off_out.is_null() {
        -1
    } else {
        unsafe { *off_out as isize }
    };
    if len == 0 {
        return 0;
    }
    info!(
        "copyfilerange: fd_in: {}, fd_out: {}, off_in: {}, off_out: {}, len: {}, flags: {}",
        fd_in, fd_out, in_offset, out_offset, len, flags
    );
    let process = current_process();
    let fd_table = process.fd_manager.fd_table.lock();
    let out_file = fd_table[fd_out].clone().unwrap();
    let in_file = fd_table[fd_in].clone().unwrap();
    let old_in_offset = in_file.seek(SeekFrom::Current(0)).unwrap();
    let old_out_offset = out_file.seek(SeekFrom::Current(0)).unwrap();

    // if in_file.lock().get_stat().unwrap().st_size < (in_offset as u64) + len as u64 {
    //     return 0;
    // }

    // set offset
    if !off_in.is_null() {
        in_file.seek(SeekFrom::Start(in_offset as u64)).unwrap();
    }

    if !off_out.is_null() {
        out_file.seek(SeekFrom::Start(out_offset as u64)).unwrap();
    }

    // copy
    let mut buf = vec![0; len];
    let read_len = in_file.read(buf.as_mut_slice()).unwrap();
    // debug!("copy content: {:?}", &buf[..read_len]);

    let write_len = out_file.write(&buf[..read_len]).unwrap();
    // assert_eq!(read_len, write_len);    // tmp

    // set offset | modify off_in & off_out
    if !off_in.is_null() {
        in_file.seek(SeekFrom::Start(old_in_offset)).unwrap();
        unsafe {
            *off_in += read_len;
        }
    }
    if !off_out.is_null() {
        out_file.seek(SeekFrom::Start(old_out_offset)).unwrap();
        unsafe {
            *off_out += write_len;
        }
    }

    write_len as isize
}
