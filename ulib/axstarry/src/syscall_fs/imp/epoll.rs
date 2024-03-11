//! The epoll API performs a similar task to poll: monitoring
//! multiple file descriptors to see if I/O is possible on any of
//! them.  
extern crate alloc;
use crate::{SyscallError, SyscallResult};
use alloc::sync::Arc;
use axhal::{mem::VirtAddr, time::current_ticks};
use axprocess::current_process;

use crate::syscall_fs::ctype::epoll::{EpollCtl, EpollEvent, EpollFile};

/// For epoll_create, Since Linux 2.6.8, the size argument is ignored, but must be greater than zero;
///
///
/// For epoll_create1, If flags is 0, then, other than the fact that the obsolete size argument is dropped, epoll_create1()
///  is the same as epoll_create().
///
/// If flag equals to EPOLL_CLOEXEC, than set the cloexec flag for the fd
/// # Arguments
/// * `flag` - usize
pub fn syscall_epoll_create1(args: [usize; 6]) -> SyscallResult {
    let _flag = args[0];
    let file = EpollFile::new();
    let process = current_process();
    let mut fd_table = process.fd_manager.fd_table.lock();
    if let Ok(num) = process.alloc_fd(&mut fd_table) {
        fd_table[num] = Some(Arc::new(file));
        Ok(num as isize)
    } else {
        // ErrorNo::EMFILE as isize
        Err(SyscallError::EMFILE)
    }
}

/// 执行syscall_epoll_ctl，修改文件对应的响应事件
///
/// 需要一个epoll事件的fd，用来执行修改操作
///
/// # Arguments
/// * `epfd`: i32, epoll文件的fd
/// * `op`: i32, 修改操作的类型
/// * `fd`: i32, 接受事件的文件的fd
/// * `event`: *const EpollEvent, 接受的事件
pub fn syscall_epoll_ctl(args: [usize; 6]) -> SyscallResult {
    let epfd = args[0] as i32;
    let op = args[1] as i32;
    let fd = args[2] as i32;
    let event = args[3] as *const EpollEvent;
    let process = current_process();
    if process.manual_alloc_type_for_lazy(event).is_err() {
        return Err(SyscallError::EFAULT);
    }
    let fd_table = process.fd_manager.fd_table.lock();
    let event = unsafe { *event };
    if fd_table[fd as usize].is_none() {
        return Err(SyscallError::EBADF);
    }
    let op = if let Ok(val) = EpollCtl::try_from(op) {
        val
    } else {
        return Err(SyscallError::EINVAL);
    };
    if let Some(file) = fd_table[epfd as usize].as_ref() {
        if let Some(epoll_file) = file.as_any().downcast_ref::<EpollFile>() {
            epoll_file.epoll_ctl(op, fd, event)
        } else {
            Err(SyscallError::EBADF)
        }
    } else {
        Err(SyscallError::EBADF)
    }
}

/// 执行syscall_epoll_wait系统调用
///
/// # Arguments
/// * `epfd`: i32, epoll文件的fd
/// * `event`: *mut EpollEvent, 接受事件的数组
/// * `max_event`: i32, 最大的响应事件数量,必须大于0
/// * `timeout`: i32, 超时时间，是一段相对时间，需要手动转化为绝对时间
///
/// ret: 实际写入的响应事件数目
pub fn syscall_epoll_wait(args: [usize; 6]) -> SyscallResult {
    let epfd = args[0] as i32;
    let event = args[1] as *mut EpollEvent;
    let max_event = args[2] as i32;
    let timeout = args[3] as i32;
    if max_event <= 0 {
        return Err(SyscallError::EINVAL);
    }
    let max_event = max_event as usize;
    let process = current_process();
    let start: VirtAddr = (event as usize).into();
    let end = start + max_event * core::mem::size_of::<EpollEvent>();
    if process.manual_alloc_range_for_lazy(start, end).is_err() {
        return Err(SyscallError::EFAULT);
    }

    let fd_table = process.fd_manager.fd_table.lock();
    let epoll_file = if let Some(file) = fd_table[epfd as usize].as_ref() {
        if let Some(epoll_file) = file.as_any().downcast_ref::<EpollFile>() {
            epoll_file.clone()
        } else {
            return Err(SyscallError::EBADF);
        }
    } else {
        return Err(SyscallError::EBADF);
    };

    let timeout = if timeout > 0 {
        current_ticks() as usize + timeout as usize
    } else {
        usize::MAX
    };
    let ret_events = epoll_file.epoll_wait(timeout);
    if ret_events.is_err() {
        return Err(SyscallError::EINTR);
    }
    let ret_events = ret_events.unwrap();
    let real_len = ret_events.len().min(max_event);
    for (i, e) in ret_events.iter().enumerate().take(real_len) {
        unsafe {
            *(event.add(i)) = *e;
        }
    }
    Ok(real_len as isize)
}
