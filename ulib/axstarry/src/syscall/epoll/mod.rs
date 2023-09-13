//! The epoll API performs a similar task to poll: monitoring
//! multiple file descriptors to see if I/O is possible on any of
//! them.  
extern crate alloc;
use alloc::sync::Arc;
use axhal::mem::VirtAddr;
use axprocess::current_process;

use self::{
    file::EpollFile,
    flags::{EpollCtl, EpollEvent},
};

use super::ErrorNo;

mod file;
pub mod flags;

/// For epoll_create, Since Linux 2.6.8, the size argument is ignored, but must be greater than zero;
///
///
/// For epoll_create1, If flags is 0, then, other than the fact that the obsolete size argument is dropped, epoll_create1()
///  is the same as epoll_create().
///
/// If flag equals to EPOLL_CLOEXEC, than set the cloexec flag for the fd
pub fn syscall_epoll_create1(_flag: usize) -> isize {
    let file = file::EpollFile::new();
    let process = current_process();
    let mut fd_table = process.fd_manager.fd_table.lock();
    if let Ok(num) = process.alloc_fd(&mut fd_table) {
        fd_table[num] = Some(Arc::new(file));
        num as isize
    } else {
        ErrorNo::EMFILE as isize
    }
}

/// 执行syscall_epoll_ctl，修改文件对应的响应事件
///
/// 需要一个epoll事件的fd，用来执行修改操作
///
/// args:
/// - epfd: epoll文件的fd
/// - op: 修改操作的类型
/// - fd: 接受事件的文件的fd
/// - event: 接受的事件
pub fn syscall_epoll_ctl(epfd: i32, op: i32, fd: i32, event: *const EpollEvent) -> isize {
    let process = current_process();
    if process
        .manual_alloc_type_for_lazy(event as *const EpollEvent)
        .is_err()
    {
        return ErrorNo::EFAULT as isize;
    }
    let fd_table = process.fd_manager.fd_table.lock();
    let event = unsafe { *event };
    if fd_table[fd as usize].is_none() {
        return ErrorNo::EBADF as isize;
    }
    let op = if let Ok(val) = EpollCtl::try_from(op) {
        val
    } else {
        return ErrorNo::EINVAL as isize;
    };
    if let Some(file) = fd_table[epfd as usize].as_ref() {
        if let Some(epoll_file) = file.as_any().downcast_ref::<EpollFile>() {
            epoll_file.epoll_ctl(op, fd, event)
        } else {
            ErrorNo::EBADF as isize
        }
    } else {
        ErrorNo::EBADF as isize
    }
}

/// 执行syscall_epoll_wait系统调用
///
/// args:
/// - epfd: epoll文件的fd
/// - event: 接受事件的数组
/// - max_event: 最大的响应事件数量，必须大于0
/// - timeout: 超时时间，是一段相对时间，需要手动转化为绝对时间
///
/// ret: 实际写入的响应事件数目
pub fn syscall_epoll_wait(
    epfd: i32,
    event: *mut EpollEvent,
    max_event: i32,
    timeout: i32,
) -> isize {
    if max_event <= 0 {
        return ErrorNo::EINVAL as isize;
    }
    let max_event = max_event as usize;
    let process = current_process();
    let start: VirtAddr = (event as usize).into();
    let end = start + max_event * core::mem::size_of::<EpollEvent>();
    if process.manual_alloc_range_for_lazy(start, end).is_err() {
        return ErrorNo::EFAULT as isize;
    }

    let fd_table = process.fd_manager.fd_table.lock();
    let epoll_file = if let Some(file) = fd_table[epfd as usize].as_ref() {
        if let Some(epoll_file) = file.as_any().downcast_ref::<EpollFile>() {
            epoll_file.clone()
        } else {
            return ErrorNo::EBADF as isize;
        }
    } else {
        return ErrorNo::EBADF as isize;
    };

    let timeout = if timeout > 0 {
        riscv::register::time::read() + timeout as usize
    } else {
        usize::MAX
    };
    let ret_events = epoll_file.epoll_wait(timeout);
    if ret_events.is_err() {
        return ErrorNo::EINTR as isize;
    }
    let ret_events = ret_events.unwrap();
    let real_len = ret_events.len().min(max_event);
    for i in 0..real_len {
        // 写入响应事件
        unsafe {
            *(event.add(i)) = ret_events[i];
        }
    }
    real_len as isize
}
