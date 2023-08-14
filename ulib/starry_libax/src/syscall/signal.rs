//! 与信号处理相关的系统调用

use axhal::cpu::this_cpu_id;
use axprocess::{
    process::{current_process, current_task, yield_now_task},
    send_signal_to_process, send_signal_to_thread,
};
use axsignal::action::SigAction;
use axsignal::signal_no::SignalNo;
use log::{debug, info};

use super::{
    flags::{SigMaskFlag, SIGSET_SIZE_IN_BYTE},
    syscall_id::ErrorNo,
};

pub fn syscall_sigaction(
    signum: usize,
    action: *const SigAction,
    old_action: *mut SigAction,
) -> isize {
    info!(
        "signum: {}, action: {:X}, old_action: {:X}",
        signum, action as usize, old_action as usize
    );
    if signum == SignalNo::SIGKILL as usize || signum == SignalNo::SIGSTOP as usize {
        // 特殊参数不能被覆盖
        return -1;
    }

    let current_process = current_process();
    let inner = current_process.inner.lock();
    let signal_module = inner
        .signal_module
        .get(&current_task().id().as_u64())
        .unwrap();
    let mut memory_set = inner.memory_set.lock();
    let mut signal_handler = signal_module.signal_handler.lock();
    let old_address = old_action as usize;

    if old_address != 0 {
        // old_address非零说明要求写入到这个地址
        // 此时要检查old_address是否在某一个段中
        if memory_set
            .manual_alloc_for_lazy(old_address.into())
            .is_err()
        {
            // 无法分配
            return -1;
        }
        if let Some(action) = signal_handler.get_action(signum) {
            // 将原有的action存储到old_address
            unsafe {
                *old_action = *action;
            }
        }
    }

    let new_address = action as usize;
    if new_address != 0 {
        if memory_set
            .manual_alloc_for_lazy(new_address.into())
            .is_err()
        {
            // 无法分配
            return -1;
        }
        signal_handler.set_action(signum, action);
    }
    0
}

/// 实现sigsuspend系统调用
pub fn syscall_sigsuspend(mask: *const usize) -> isize {
    let process = current_process();
    let mut inner = process.inner.lock();
    if inner
        .memory_set
        .lock()
        .manual_alloc_for_lazy((mask as usize).into())
        .is_err()
    {
        return ErrorNo::EFAULT as isize;
    }
    let signal_module = inner
        .signal_module
        .get_mut(&current_task().id().as_u64())
        .unwrap();
    // 设置新的掩码
    if signal_module.last_trap_frame_for_signal.is_some() {
        // 信号嵌套的情况下触发这个调用
        return ErrorNo::EINTR as isize;
    }
    signal_module.signal_set.mask = unsafe { *mask };
    drop(inner);
    loop {
        let mut inner = process.inner.lock();
        let signal_module = inner
            .signal_module
            .get_mut(&current_task().id().as_u64())
            .unwrap();
        if signal_module.signal_set.find_signal().is_none() {
            drop(inner);
            // 记得释放锁
            yield_now_task();
        }
        break;
    }
    return -1;
}

pub fn syscall_sigreturn() -> isize {
    axprocess::signal_return()
}

pub fn syscall_sigprocmask(
    flag: SigMaskFlag,
    new_mask: *const usize,
    old_mask: *mut usize,
    sigsetsize: usize,
) -> isize {
    if sigsetsize != SIGSET_SIZE_IN_BYTE {
        // 若sigsetsize不是正确的大小，则返回错误
        return ErrorNo::EINVAL as isize;
    }

    let current_process = current_process();
    let mut inner = current_process.inner.lock();

    let mut memory_set = inner.memory_set.lock();
    if old_mask as usize != 0
        && memory_set
            .manual_alloc_for_lazy((old_mask as usize).into())
            .is_err()
    {
        return ErrorNo::EFAULT as isize;
    }
    if new_mask as usize != 0
        && memory_set
            .manual_alloc_for_lazy((new_mask as usize).into())
            .is_err()
    {
        return -1;
    }

    drop(memory_set);
    let signal_module = inner
        .signal_module
        .get_mut(&current_task().id().as_u64())
        .unwrap();
    if old_mask as usize != 0 {
        unsafe {
            *old_mask = signal_module.signal_set.mask;
        }
    }

    if new_mask as usize != 0 {
        let now_mask = unsafe { *new_mask };
        match flag {
            SigMaskFlag::SigBlock => {
                signal_module.signal_set.mask |= now_mask;
            }
            SigMaskFlag::SigUnblock => {
                signal_module.signal_set.mask &= !now_mask;
            }
            SigMaskFlag::SigSetmask => {
                signal_module.signal_set.mask = now_mask;
            }
        }
    }
    0
}

/// 向pid指定的进程发送信号
///
/// 由于处理信号的单位在线程上，所以若进程中有多个线程，则会发送给主线程
pub fn syscall_kill(pid: isize, signum: isize) -> isize {
    if pid > 0 && signum > 0 {
        // 不关心是否成功
        let _ = send_signal_to_process(pid, signum);
        0
    } else if pid == 0 {
        ErrorNo::ESRCH as isize
    } else {
        ErrorNo::EINVAL as isize
    }
}

/// 向tid指定的线程发送信号
pub fn syscall_tkill(tid: isize, signum: isize) -> isize {
    debug!(
        "cpu: {}, send singal: {} to: {}",
        this_cpu_id(),
        signum,
        tid
    );
    if tid > 0 && signum > 0 {
        let _ = send_signal_to_thread(tid, signum);
        0
    } else {
        ErrorNo::EINVAL as isize
    }
}
