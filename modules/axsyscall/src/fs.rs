use alloc::sync::Arc;
use log::debug;
/// 处理关于输出输入的系统调用
use axfs_os::new_fd;
use axfs_os::pipe::make_pipe;
use axprocess::process::current_process;

#[allow(unused)]
// const STDIN: usize = 0;
// const STDOUT: usize = 1;
// const STDERR: usize = 2;
pub fn syscall_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner.lock();
    if fd >= process_inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = process_inner.fd_table[fd].as_ref() {
        if !file.readable() {
            return -1;
        }
        let file = file.clone();
        drop(process_inner); // release current inner manually to avoid multi-borrow
        file.read(unsafe { core::slice::from_raw_parts_mut(buf, len) })
            .unwrap() as isize
    } else {
        -1
    }
}

pub fn syscall_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner.lock();
    if fd >= process_inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = process_inner.fd_table[fd].as_ref() {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        drop(process_inner); // release current inner manually to avoid multi-borrow
        // file.write("Test SysWrite\n".as_bytes()).unwrap();
        file.write(unsafe { core::slice::from_raw_parts(buf, len) })
            .unwrap() as isize
    } else {
        -1
    }
}

/// 打开文件, flags由低到高位: read, write, append, truncate, create, create_new
/// _dir_fd暂时不用，path仅支持绝对路径，flags和_mode格式待考察
pub fn syscall_open(_dir_fd: usize, path: *const u8, flags: u8, _mode: u8) -> isize {
    // 从path指向的字符串中读取文件名
    let raw_str = unsafe { core::slice::from_raw_parts(path, 256) };
    let mut len = 0 as usize;
    for i in 0..256 {
        if raw_str[i] == 0 {
            len = i;
            break;
        }
    }
    let path = &raw_str[0..len];
    let path = core::str::from_utf8(path).unwrap();

    let process = current_process();
    let mut process_inner = process.inner.lock();

    let fd_num = process_inner.alloc_fd();
    if let Ok(file) = new_fd(path, flags) {
        process_inner.fd_table[fd_num] = Some(Arc::new(file));
        fd_num as isize
    } else {
        debug!("open file failed");
        -1
    }
}
// pub fn syscall_open(path: *const u8, len: usize, flags: u8) -> isize {
//     let path = unsafe { core::slice::from_raw_parts(path, len) };
//     let path = core::str::from_utf8(path).unwrap();
//
//     let process = current_process();
//     let mut process_inner = process.inner.lock();
//
//     let fd_num = process_inner.alloc_fd();
//     if let Ok(file) = new_fd(path, flags) {
//         process_inner.fd_table[fd_num] = Some(Arc::new(file));
//         fd_num as isize
//     } else {
//         -1
//     }
// }

pub fn syscall_close(fd: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner.lock();
    if fd >= process_inner.fd_table.len() {
        debug!("fd {} is out of range", fd);
        return -1;
    }
    if fd == 3 {
        debug!("fd {} is reserved for cwd", fd);
        return -1;
    }
    if process_inner.fd_table[fd].is_none() {
        debug!("fd {} is not opened", fd);
        return -1;
    }
    process_inner.fd_table[fd].take();
    0
}


pub fn syscall_getcwd(mut buf: *mut u8, len: usize) -> * u8 {
    let process = current_process();
    let process_inner = process.inner.lock();
    let cwd = process_inner.get_cwd();

    // todo: 如果buf为NULL,则系统分配缓存区
    // if buf.is_null() {
    //     buf = allocate_buffer(cwd.len());   // 分配缓存区 allocate_buffer待实现
    // }

    let cwd = cwd.as_bytes();
    let len = core::cmp::min(len, cwd.len());
    unsafe {
        core::ptr::copy_nonoverlapping(cwd.as_ptr(), buf, len);
    }
    // 返回当前工作目录的字符串的指针
    if len == cwd.len() {   // 如果buf的长度足够大
        buf as * u8
    } else {
        debug!("getcwd: buf size is too small");
        core::ptr::null()
    }
}

pub fn syscall_pipe2(fd: *mut usize) -> isize {        // 传入的fd是一个数组fd[2]的地址,fd[0]为管道的读出端，fd[1]为管道的写入端。
    let process = current_process();
    let mut process_inner = process.inner.lock();

    let fd_num = process_inner.alloc_fd();
    let fd_num2 = process_inner.alloc_fd();

    let (read, write) = make_pipe();
    process_inner.fd_table[fd_num] = Some(read);
    process_inner.fd_table[fd_num2] = Some(write);

    unsafe {
        core::ptr::write(fd, fd_num as usize);
        core::ptr::write(fd.offset(1), fd_num2 as usize);
    }
    0
}


