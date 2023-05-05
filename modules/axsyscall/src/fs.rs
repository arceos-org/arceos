use alloc::sync::Arc;
/// 处理关于输出输入的系统调用
use axfs_os::new_fd;
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
        return -1;
    }
    if process_inner.fd_table[fd].is_none() {
        return -1;
    }
    process_inner.fd_table[fd].take();
    0
}
