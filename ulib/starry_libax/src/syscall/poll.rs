//! 用于实现poll/ppoll相关的系统调用
extern crate alloc;
use alloc::vec::Vec;
use axprocess::process::{current_process, yield_now_task};
use bitflags::bitflags;
use memory_addr::VirtAddr;

use super::{flags::TimeSecs, syscall_id::ErrorNo};
bitflags! {
    /// 在文件上等待或者发生过的事件
    #[derive(Clone, Copy,Debug)]
    pub struct PollEvents: u16 {
        /// 可读
        const IN = 0x0001;
        /// 可写
        const OUT = 0x0004;
        /// 错误
        const ERR = 0x0008;
        /// 挂起，如pipe另一端关闭
        const HUP = 0x0010;
        /// 无效的事件
        const NVAL = 0x0020;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PollFd {
    /// 等待的fd
    pub fd: i32,
    /// 等待的事件
    pub events: PollEvents,
    /// 返回的事件
    pub revents: PollEvents,
}

/// 实现ppoll系统调用
///
/// fds：一个PollFd列表
/// expire_time：时间戳，用来记录是否超时
///
/// 返回值：(usize, Vec<PollFd>) 第一个参数遵守 ppoll 系统调用的返回值约定，第二个参数为返回的 `PollFd` 列表
fn ppoll(mut fds: Vec<PollFd>, expire_time: usize) -> (usize, Vec<PollFd>) {
    loop {
        // 满足事件要求而被触发的事件描述符数量
        let mut set: usize = 0;
        let current_process = current_process();
        let inner = current_process.inner.lock();
        for poll_fd in &mut fds {
            if let Some(file) = inner.fd_manager.fd_table[poll_fd.fd as usize].as_ref() {
                poll_fd.revents = PollEvents::empty();
                // let file = file.lock();
                if file.lock().in_exceptional_conditions() {
                    poll_fd.revents |= PollEvents::ERR;
                }
                if file.lock().is_hang_up() {
                    poll_fd.revents |= PollEvents::HUP;
                }
                if poll_fd.events.contains(PollEvents::IN) && file.lock().ready_to_read() {
                    poll_fd.revents |= PollEvents::IN;
                }
                if poll_fd.events.contains(PollEvents::OUT) && file.lock().ready_to_write() {
                    poll_fd.revents |= PollEvents::OUT;
                }
                // 如果返回事件不为空，代表有响应
                if !poll_fd.revents.is_empty() {
                    set += 1;
                }
            } else {
                // 不存在也是一种响应
                poll_fd.revents = PollEvents::ERR;
                set += 1;
            }
        }
        if set > 0 {
            return (set, fds);
        }
        if riscv::register::time::read() > expire_time {
            // 过期了，直接返回
            return (0, fds);
        }
        yield_now_task();
    }
}

/// 实现ppoll系统调用
///
/// 其中timeout是一段相对时间，需要计算出相对于当前时间戳的绝对时间戳
pub fn syscall_ppoll(
    ufds: *mut PollFd,
    nfds: usize,
    timeout: *const TimeSecs,
    _mask: usize,
) -> isize {
    let process = current_process();
    let inner = process.inner.lock();
    let start: VirtAddr = (ufds as usize).into();
    let end = start + nfds * core::mem::size_of::<PollFd>();
    if inner
        .memory_set
        .lock()
        .manual_alloc_range_for_lazy(start, end)
        .is_err()
    {
        return ErrorNo::EFAULT as isize;
    }

    let mut fds: Vec<PollFd> = Vec::new();

    for i in 0..nfds {
        unsafe {
            fds.push(*(ufds.add(i)));
        }
    }

    let expire_time = if timeout as usize != 0 {
        if inner
            .memory_set
            .lock()
            .manual_alloc_type_for_lazy(timeout)
            .is_err()
        {
            return ErrorNo::EFAULT as isize;
        }
        riscv::register::time::read() + unsafe { (*timeout).get_ticks() }
    } else {
        usize::MAX
    };

    drop(inner);
    let (set, ret_fds) = ppoll(fds, expire_time);
    // 将得到的fd存储到原先的指针中
    for i in 0..ret_fds.len() {
        unsafe {
            *(ufds.add(i)) = ret_fds[i];
        }
    }
    set as isize
}
