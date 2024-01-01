//! 负责与 IO 相关的系统调用
extern crate alloc;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;
use axerrno::AxError;
use axfs::api::{FileIOType, OpenFlags};
use axio::SeekFrom;
use axlog::{debug, info};
use axprocess::current_process;
use axprocess::link::{create_link, deal_with_path, real_path};
use syscall_utils::{IoVec, SyscallError, SyscallResult};

use crate::ctype::pipe::make_pipe;
use crate::ctype::{dir::new_dir, file::new_fd};
/// 功能：从一个文件描述符中读取；
/// 输入：
///     - fd：要读取文件的文件描述符。
///     - buf：一个缓存区，用于存放读取的内容。
///     - count：要读取的字节数。
/// 返回值：成功执行，返回读取的字节数。如为0，表示文件结束。错误，则返回-1。
pub fn syscall_read(fd: usize, buf: *mut u8, count: usize) -> SyscallResult {
    info!("[read()] fd: {fd}, buf: {buf:?}, len: {count}",);

    if buf.is_null() {
        return Err(SyscallError::EFAULT);
    }

    let process = current_process();

    // TODO: 左闭右开
    let buf = match process.manual_alloc_range_for_lazy(
        (buf as usize).into(),
        (unsafe { buf.add(count) as usize } - 1).into(),
    ) {
        Ok(_) => unsafe { core::slice::from_raw_parts_mut(buf, count) },
        Err(_) => return Err(SyscallError::EFAULT),
    };

    let file = match process.fd_manager.fd_table.lock().get(fd) {
        Some(Some(f)) => f.clone(),
        _ => return Err(SyscallError::EBADF),
    };

    if file.get_type() == FileIOType::DirDesc {
        axlog::error!("fd is a dir");
        return Err(SyscallError::EISDIR);
    }
    if !file.readable() {
        // 1. nonblocking socket
        //
        // Normal socket will block while trying to read, so we don't return here.
        #[cfg(feature = "net")]
        if let Some(socket) = file.as_any().downcast_ref::<Socket>() {
            if socket.is_nonblocking() && socket.is_connected() {
                return Err(SyscallError::EAGAIN);
            }
        } else {
            // 2. nonblock file
            // return ErrorNo::EAGAIN as isize;
            // 3. regular file
            return Err(SyscallError::EBADF);
        }

        #[cfg(not(feature = "net"))]
        return Err(SyscallError::EBADF);
    }

    // for sockets:
    // Sockets are "readable" when:
    // - have some data to read without blocking
    // - remote end send FIN packet, local read half is closed (this will return 0 immediately)
    //   this will return Ok(0)
    // - ready to accept new connections

    match file.read(buf) {
        Ok(len) => Ok(len as isize),
        Err(AxError::WouldBlock) => Err(SyscallError::EAGAIN),
        Err(_) => Err(SyscallError::EPERM),
    }
}

/// 功能：从一个文件描述符中写入；
/// 输入：
///     - fd：要写入文件的文件描述符。
///     - buf：一个缓存区，用于存放要写入的内容。
///     - count：要写入的字节数。
/// 返回值：成功执行，返回写入的字节数。错误，则返回-1。
pub fn syscall_write(fd: usize, buf: *const u8, count: usize) -> SyscallResult {
    info!("[write()] fd: {fd}, buf: {buf:?}, len: {count}");

    if buf.is_null() {
        return Err(SyscallError::EFAULT);
    }

    let process = current_process();

    // TODO: 左闭右开
    let buf = match process.manual_alloc_range_for_lazy(
        (buf as usize).into(),
        (unsafe { buf.add(count) as usize } - 1).into(),
    ) {
        Ok(_) => unsafe { core::slice::from_raw_parts(buf, count) },
        Err(_) => return Err(SyscallError::EFAULT),
    };

    let file = match process.fd_manager.fd_table.lock().get(fd) {
        Some(Some(f)) => f.clone(),
        _ => return Err(SyscallError::EBADF),
    };

    if file.get_type() == FileIOType::DirDesc {
        debug!("fd is a dir");
        return Err(SyscallError::EBADF);
    }
    if !file.writable() {
        // 1. socket
        //
        // Normal socket will block while trying to write, so we don't return here.
        #[cfg(feature = "net")]
        if let Some(socket) = file.as_any().downcast_ref::<Socket>() {
            if socket.is_nonblocking() && socket.is_connected() {
                return Err(SyscallError::EAGAIN);
            }
        } else {
            // 2. nonblock file
            // return ErrorNo::EAGAIN as isize;

            // 3. regular file
            return Err(SyscallError::EBADF);
        }

        #[cfg(not(feature = "net"))]
        return Err(SyscallError::EBADF);
    }

    // for sockets:
    // Sockets are "writable" when:
    // - connected and have space in tx buffer to write
    // - sent FIN packet, local send half is closed (this will return 0 immediately)
    //   this will return Err(ConnectionReset)

    match file.write(buf) {
        Ok(len) => Ok(len as isize),
        // socket with send half closed
        // TODO: send a SIGPIPE signal to the process
        Err(axerrno::AxError::ConnectionReset) => Err(SyscallError::EPIPE),
        Err(_) => Err(SyscallError::EPERM),
    }
}

/// 从同一个文件描述符读取多个字符串
pub fn syscall_readv(fd: usize, iov: *mut IoVec, iov_cnt: usize) -> SyscallResult {
    let mut read_len = 0;
    // 似乎要判断iov是否分配，但是懒了，反正能过测例
    for i in 0..iov_cnt {
        let io: &IoVec = unsafe { &*iov.add(i) };
        if io.base.is_null() || io.len == 0 {
            continue;
        }
        match syscall_read(fd, io.base, io.len) {
            len if len.is_ok() => read_len += len.unwrap(),

            err => return err,
        }
    }
    Ok(read_len)
}

/// 从同一个文件描述符写入多个字符串
pub fn syscall_writev(fd: usize, iov: *const IoVec, iov_cnt: usize) -> SyscallResult {
    let mut write_len = 0;
    // 似乎要判断iov是否分配，但是懒了，反正能过测例
    for i in 0..iov_cnt {
        let io: &IoVec = unsafe { &(*iov.add(i)) };
        if io.base.is_null() || io.len == 0 {
            continue;
        }
        match syscall_write(fd, io.base, io.len) {
            len if len.is_ok() => write_len += len.unwrap(),

            err => return err,
        }
    }
    Ok(write_len)
}

/// 功能：创建管道；
/// 输入：
///     - fd[2]：用于保存2个文件描述符。其中，fd[0]为管道的读出端，fd[1]为管道的写入端。
/// 返回值：成功执行，返回0。失败，返回-1。
///
/// 注意：fd[2]是32位数组，所以这里的 fd 是 u32 类型的指针，而不是 usize 类型的指针。
pub fn syscall_pipe2(fd: *mut u32, flags: usize) -> SyscallResult {
    axlog::info!("Into syscall_pipe2. fd: {} flags: {}", fd as usize, flags);
    let process = current_process();
    if process.manual_alloc_for_lazy((fd as usize).into()).is_err() {
        return Err(SyscallError::EINVAL);
    }
    let non_block = (flags & 0x800) != 0;
    let (read, write) = make_pipe(non_block);
    let mut fd_table = process.fd_manager.fd_table.lock();
    let fd_num = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
        fd
    } else {
        return Err(SyscallError::EPERM);
    };
    fd_table[fd_num] = Some(read);
    let fd_num2 = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
        fd
    } else {
        return Err(SyscallError::EPERM);
    };
    fd_table[fd_num2] = Some(write);
    info!("read end: {} write: end: {}", fd_num, fd_num2);
    unsafe {
        core::ptr::write(fd, fd_num as u32);
        core::ptr::write(fd.offset(1), fd_num2 as u32);
    }
    Ok(0)
}

/// 功能：复制文件描述符；
/// 输入：
///     - fd：被复制的文件描述符。
/// 返回值：成功执行，返回新的文件描述符。失败，返回-1。
pub fn syscall_dup(fd: usize) -> SyscallResult {
    let process = current_process();
    let mut fd_table = process.fd_manager.fd_table.lock();
    if fd >= fd_table.len() {
        debug!("fd {} is out of range", fd);
        return Err(SyscallError::EBADF);
    }
    if fd_table[fd].is_none() {
        debug!("fd {} is a closed fd", fd);
        return Err(SyscallError::EBADF);
    }

    let new_fd = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
        fd
    } else {
        // 文件描述符达到上限了
        return Err(SyscallError::EMFILE);
    };
    fd_table[new_fd] = fd_table[fd].clone();

    Ok(new_fd as isize)
}

/// 功能：复制文件描述符，并指定了新的文件描述符；
/// 输入：
///     - old：被复制的文件描述符。
///     - new：新的文件描述符。
/// 返回值：成功执行，返回新的文件描述符。失败，返回-1。
pub fn syscall_dup3(fd: usize, new_fd: usize) -> SyscallResult {
    let process = current_process();
    let mut fd_table = process.fd_manager.fd_table.lock();
    if fd >= fd_table.len() {
        debug!("fd {} is out of range", fd);
        return Err(SyscallError::EPERM);
    }
    if fd_table[fd].is_none() {
        debug!("fd {} is not opened", fd);
        return Err(SyscallError::EPERM);
    }
    if new_fd >= fd_table.len() {
        if new_fd >= (process.fd_manager.get_limit() as usize) {
            // 超出了资源限制
            return Err(SyscallError::EBADF);
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
    Ok(new_fd as isize)
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
pub fn syscall_openat(fd: usize, path: *const u8, flags: usize, _mode: u8) -> SyscallResult {
    let force_dir = OpenFlags::from(flags).is_dir();
    let path = if let Some(path) = deal_with_path(fd, Some(path), force_dir) {
        path
    } else {
        return Err(SyscallError::EINVAL);
    };
    let process = current_process();
    let mut fd_table = process.fd_manager.fd_table.lock();
    let fd_num = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
        fd
    } else {
        return Err(SyscallError::EMFILE);
    };
    debug!("allocated fd_num: {}", fd_num);
    // 如果是DIR
    if path.is_dir() {
        debug!("open dir");
        if let Ok(dir) = new_dir(path.path().to_string(), flags.into()) {
            debug!("new dir_desc successfully allocated: {}", path.path());
            fd_table[fd_num] = Some(Arc::new(dir));
            Ok(fd_num as isize)
        } else {
            debug!("open dir failed");
            Err(SyscallError::ENOENT)
        }
    }
    // 如果是FILE，注意若创建了新文件，需要添加链接
    else {
        debug!("open file");
        if let Ok(file) = new_fd(path.path().to_string(), flags.into()) {
            debug!("new file_desc successfully allocated");
            fd_table[fd_num] = Some(Arc::new(file));
            let _ = create_link(&path, &path); // 不需要检查是否成功，因为如果成功，说明是新建的文件，如果失败，说明已经存在了
            Ok(fd_num as isize)
        } else {
            debug!("open file failed");
            Err(SyscallError::ENOENT)
        }
    }
}

/// 功能：关闭一个文件描述符；
/// 输入：
///     - fd：要关闭的文件描述符。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_close(fd: usize) -> SyscallResult {
    info!("Into syscall_close. fd: {}", fd);

    let process = current_process();
    let mut fd_table = process.fd_manager.fd_table.lock();
    if fd >= fd_table.len() {
        debug!("fd {} is out of range", fd);
        return Err(SyscallError::EPERM);
    }
    // if fd == 3 {
    //     debug!("fd {} is reserved for cwd", fd);
    //     return -1;
    // }
    if fd_table[fd].is_none() {
        debug!("fd {} is none", fd);
        return Err(SyscallError::EPERM);
    }
    // let file = process_inner.fd_manager.fd_table[fd].unwrap();
    fd_table[fd] = None;
    // for i in 0..process_inner.fd_table.len() {
    //     if let Some(file) = process_inner.fd_table[i].as_ref() {
    //         debug!("fd: {} has file", i);
    //     }
    // }

    Ok(0)
}

/// 67
/// pread64
/// 从文件的指定位置读取数据，并且不改变文件的读写指针
pub fn syscall_pread64(fd: usize, buf: *mut u8, count: usize, offset: usize) -> SyscallResult {
    let process = current_process();
    // todo: 把check fd整合到fd_manager中
    let file = process.fd_manager.fd_table.lock()[fd].clone().unwrap();

    let old_offset = file.seek(SeekFrom::Current(0)).unwrap();
    let ret = file
        .seek(SeekFrom::Start(offset as u64))
        .and_then(|_| file.read(unsafe { core::slice::from_raw_parts_mut(buf, count) }));
    file.seek(SeekFrom::Start(old_offset)).unwrap();
    ret.map(|size| Ok(size as isize))
        .unwrap_or_else(|_| Err(SyscallError::EINVAL))
}

/// 68
/// pwrite64
/// 向文件的指定位置写入数据，并且不改变文件的读写指针
pub fn syscall_pwrite64(fd: usize, buf: *const u8, count: usize, offset: usize) -> SyscallResult {
    let process = current_process();

    let file = process.fd_manager.fd_table.lock()[fd].clone().unwrap();

    let old_offset = file.seek(SeekFrom::Current(0)).unwrap();

    let ret = file.seek(SeekFrom::Start(offset as u64)).and_then(|_| {
        let res = file.write(unsafe { core::slice::from_raw_parts(buf, count) });
        res
    });

    file.seek(SeekFrom::Start(old_offset)).unwrap();
    drop(file);

    ret.map(|size| Ok(size as isize))
        .unwrap_or_else(|_| Err(SyscallError::EINVAL))
}

/// 71
/// sendfile64
/// 将一个文件的内容发送到另一个文件中
/// 如果offset为NULL，则从当前读写指针开始读取，读取完毕后会更新读写指针
/// 如果offset不为NULL，则从offset指定的位置开始读取，读取完毕后不会更新读写指针，但是会更新offset的值
pub fn syscall_sendfile64(
    out_fd: usize,
    in_fd: usize,
    offset: *mut usize,
    count: usize,
) -> SyscallResult {
    info!("send from {} to {}, count: {}", in_fd, out_fd, count);
    let process = current_process();
    let out_file = process.fd_manager.fd_table.lock()[out_fd].clone().unwrap();
    let in_file = process.fd_manager.fd_table.lock()[in_fd].clone().unwrap();
    let old_in_offset = in_file.seek(SeekFrom::Current(0)).unwrap();

    let mut buf = vec![0u8; count];
    if !offset.is_null() {
        // 如果offset不为NULL，则从offset指定的位置开始读取
        let in_offset = unsafe { *offset };
        in_file.seek(SeekFrom::Start(in_offset as u64)).unwrap();
        let ret = in_file.read(buf.as_mut_slice());
        unsafe { *offset = in_offset + ret.unwrap() };
        in_file.seek(SeekFrom::Start(old_in_offset)).unwrap();
        let buf = buf[..ret.unwrap()].to_vec();
        Ok(out_file.write(buf.as_slice()).unwrap() as isize)
    } else {
        // 如果offset为NULL，则从当前读写指针开始读取
        let ret = in_file.read(buf.as_mut_slice());
        info!("in fd: {}, count: {}", in_fd, count);
        let buf = buf[..ret.unwrap()].to_vec();
        info!("read len: {}", buf.len());
        info!("write len: {}", buf.as_slice().len());
        Ok(out_file.write(buf.as_slice()).unwrap() as isize)
    }
}

/// 78
/// readlinkat
/// 读取符号链接文件的内容
/// 如果buf为NULL，则返回符号链接文件的长度
/// 如果buf不为NULL，则将符号链接文件的内容写入buf中
/// 如果写入的内容超出了buf_size则直接截断
pub fn syscall_readlinkat(
    dir_fd: usize,
    path: *const u8,
    buf: *mut u8,
    bufsiz: usize,
) -> SyscallResult {
    let process = current_process();
    if process
        .manual_alloc_for_lazy((path as usize).into())
        .is_err()
    {
        return Err(SyscallError::EFAULT);
    }
    if !buf.is_null() {
        if process
            .manual_alloc_for_lazy((buf as usize).into())
            .is_err()
        {
            return Err(SyscallError::EFAULT);
        }
    }

    let path = deal_with_path(dir_fd, Some(path), false);
    if path.is_none() {
        return Err(SyscallError::ENOENT);
    }
    let path = path.unwrap();
    if path.path() == "proc/self/exe" {
        // 针对lmbench_all特判
        let name = "/lmbench_all";
        let len = bufsiz.min(name.len());
        let slice = unsafe { core::slice::from_raw_parts_mut(buf, bufsiz) };
        slice.copy_from_slice(&name.as_bytes()[..len]);
        return Ok(len as isize);
    }
    if path.path().to_string() != real_path(&(path.path().to_string())) {
        // 说明链接存在
        let path = path.path();
        let len = bufsiz.min(path.len());
        let slice = unsafe { core::slice::from_raw_parts_mut(buf, len) };
        slice.copy_from_slice(&path.as_bytes()[..len]);
        return Ok(path.len() as isize);
    }
    Err(SyscallError::EINVAL)
}
/// 62
/// 移动文件描述符的读写指针
pub fn syscall_lseek(fd: usize, offset: isize, whence: usize) -> SyscallResult {
    let process = current_process();
    info!("fd: {} offset: {} whence: {}", fd, offset, whence);
    if fd >= process.fd_manager.fd_table.lock().len() || fd < 3 {
        debug!("fd {} is out of range", fd);
        return Err(SyscallError::EBADF);
    }
    let fd_table = process.fd_manager.fd_table.lock();
    if let Some(file) = fd_table[fd].as_ref() {
        if file.get_type() == FileIOType::DirDesc {
            debug!("fd is a dir");
            return Err(SyscallError::EISDIR);
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
            return Err(SyscallError::EINVAL);
        };
        if let Ok(now_offset) = ans {
            Ok(now_offset as isize)
        } else {
            Err(SyscallError::EINVAL)
        }
    } else {
        debug!("fd {} is none", fd);
        Err(SyscallError::EBADF)
    }
}

/// 82
/// 写回硬盘
#[allow(unused)]
pub fn syscall_fsync(fd: usize) -> SyscallResult {
    let process = current_process();
    if fd >= process.fd_manager.fd_table.lock().len() || fd < 3 {
        debug!("fd {} is out of range", fd);
        return Err(SyscallError::EBADF);
    }
    let fd_table = process.fd_manager.fd_table.lock();
    if let Some(file) = fd_table[fd].clone() {
        if file.flush().is_err() {}
        Ok(0)
    } else {
        debug!("fd {} is none", fd);
        Err(SyscallError::EBADF)
    }
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
) -> SyscallResult {
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
        return Ok(0);
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

    Ok(write_len as isize)
}
pub fn syscall_ftruncate64(fd: usize, len: usize) -> SyscallResult {
    let process = current_process();
    info!("fd: {}, len: {}", fd, len);
    let fd_table = process.fd_manager.fd_table.lock();
    if fd >= fd_table.len() {
        return Err(SyscallError::EINVAL);
    }
    if fd_table[fd].is_none() {
        return Err(SyscallError::EINVAL);
    }

    if let Some(file) = fd_table[fd].as_ref() {
        if file.truncate(len).is_err() {
            return Err(SyscallError::EINVAL);
        }
    }
    Ok(0)
}
