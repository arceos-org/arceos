//! 支持 futex 相关的 syscall

extern crate alloc;
use alloc::collections::VecDeque;
use axhal::mem::VirtAddr;
use axlog::info;
use axprocess::{
    current_process, current_task,
    futex::{FutexRobustList, FUTEX_WAIT_TASK, WAIT_FOR_FUTEX},
    yield_now_task,
};
use axtask::TaskState;

use crate::{FutexFlags, RobustList, SyscallError, SyscallResult, TimeSecs};

/// Futex requeue操作
///
/// 首先唤醒src_addr对应的futex变量的等待队列中，至多wake_num个任务
///
/// 若原队列中的任务数大于wake_num，则将多余的任务移动到dst_addr对应的futex变量的等待队列中
///
/// 移动的任务数目至多为move_num
///
/// 不考虑检查操作
pub fn futex_requeue(wake_num: u32, move_num: usize, src_addr: VirtAddr, dst_addr: VirtAddr) {
    let mut futex_wait_task = FUTEX_WAIT_TASK.lock();
    if !futex_wait_task.contains_key(&src_addr) {
        return;
    }
    let src_wait_task = futex_wait_task.get_mut(&src_addr).unwrap();
    for _ in 0..wake_num {
        if let Some((task, _)) = src_wait_task.pop_front() {
            WAIT_FOR_FUTEX.notify_task(false, &task);
        } else {
            break;
        }
    }

    if !src_wait_task.is_empty() {
        let move_num = move_num.min(src_wait_task.len());

        let mut temp_move_task = src_wait_task.drain(..move_num).collect::<VecDeque<_>>();
        let dst_wait_task = futex_wait_task.entry(dst_addr).or_default();
        dst_wait_task.append(&mut temp_move_task);
    }
}

/// To do the futex operation
///
/// It may create, remove the futex wait task or requeue the futex wait task
pub fn futex(
    vaddr: VirtAddr,
    futex_op: i32,
    val: u32,
    timeout: usize,
    vaddr2: VirtAddr,
    _val3: u32,
) -> Result<usize, SyscallError> {
    let flag = FutexFlags::new(futex_op);
    let current_task = current_task();
    match flag {
        FutexFlags::Wait => {
            let process = current_process();
            if process.manual_alloc_for_lazy(vaddr).is_ok() {
                let real_futex_val = unsafe { (vaddr.as_usize() as *const u32).read_volatile() };
                info!("real val: {:#x}, expected val: {:#x}", real_futex_val, val);
                if real_futex_val != val {
                    return Err(SyscallError::EAGAIN);
                }
                let mut futex_wait_task = FUTEX_WAIT_TASK.lock();
                let wait_list = futex_wait_task.entry(vaddr).or_default();
                wait_list.push_back((current_task.as_task_ref().clone(), val));
                // // 输出每一个键值对应的vec的长度
                drop(futex_wait_task);
                // info!("timeout: {}", timeout as u64);
                // debug!("ready wait!");
                if timeout == 0 {
                    yield_now_task();
                }
                #[cfg(feature = "signal")]
                {
                    use core::time::Duration;
                    if timeout != 0
                        && !WAIT_FOR_FUTEX.wait_timeout(Duration::from_nanos(timeout as u64))
                        && process.have_signals().is_some()
                    {
                        // 被信号打断
                        return Err(SyscallError::EINTR);
                    }
                }

                Ok(0)
            } else {
                Err(SyscallError::EFAULT)
            }
        }
        FutexFlags::Wake => {
            // // 当前任务释放了锁，所以不需要再次释放
            let mut futex_wait_task = FUTEX_WAIT_TASK.lock();
            if futex_wait_task.contains_key(&vaddr) {
                let wait_list = futex_wait_task.get_mut(&vaddr).unwrap();
                // info!("now task: {}", wait_list.len());
                loop {
                    if let Some((task, _)) = wait_list.pop_front() {
                        // 唤醒一个正在等待的任务
                        if task.state() != TaskState::Blocked {
                            // 说明自己已经醒了，那么就不在wait里面了
                            continue;
                        }
                        info!("wake task: {}", task.id().as_u64());
                        drop(futex_wait_task);
                        WAIT_FOR_FUTEX.notify_task(false, &task);
                    } else {
                        drop(futex_wait_task);
                    }
                    break;
                }
            } else {
                drop(futex_wait_task);
            }
            yield_now_task();
            Ok(val as usize)
        }
        FutexFlags::Requeue => {
            // 此时timeout相当于val2，即是move_num
            futex_requeue(val, timeout, vaddr, vaddr2);
            Ok(0)
        }
        _ => {
            Err(SyscallError::EINVAL)
            // return Ok(0);
        }
    }
}

/// To check the futex wait task
///
/// If the futex value has been changed, then wake up the task
pub fn check_dead_wait() {
    let process = current_process();
    let mut futex_wait_task = FUTEX_WAIT_TASK.lock();
    for (vaddr, wait_list) in futex_wait_task.iter_mut() {
        if process.manual_alloc_for_lazy(*vaddr).is_ok() {
            let real_futex_val = unsafe { ((*vaddr).as_usize() as *const u32).read_volatile() };
            for (task, val) in wait_list.iter() {
                if real_futex_val != *val && task.state() == TaskState::Blocked {
                    WAIT_FOR_FUTEX.notify_task(false, task);
                }
            }
            // 仅保留那些真正等待的任务
            wait_list
                .retain(|(task, val)| real_futex_val == *val && task.state() == TaskState::Blocked);
        }
    }
}

/// # Arguments
/// * vaddr: usize
/// * futex_op: i32
/// * futex_val: u32
/// * time_out_val: usize
/// * vaddr2: usize
/// * val3: u32
pub fn syscall_futex(args: [usize; 6]) -> SyscallResult {
    let vaddr = args[0];
    let futex_op = args[1] as i32;
    let futex_val = args[2] as u32;
    let time_out_val = args[3];
    let vaddr2 = args[4];
    let val3 = args[5] as u32;
    let process = current_process();
    let timeout = if time_out_val != 0 && process.manual_alloc_for_lazy(time_out_val.into()).is_ok()
    {
        let time_sepc: TimeSecs = unsafe { *(time_out_val as *const TimeSecs) };
        time_sepc.turn_to_nanos()
    } else {
        // usize::MAX
        0
    };
    // 释放锁，防止任务无法被调度
    match futex(
        vaddr.into(),
        futex_op,
        futex_val,
        timeout,
        vaddr2.into(),
        val3,
    ) {
        Ok(ans) => Ok(ans as isize),
        Err(errno) => Err(errno),
    }
}

/// 内核只发挥存储的作用
/// 但要保证head对应的地址已经被分配
/// # Arguments
/// * head: usize
/// * len: usize
pub fn syscall_set_robust_list(args: [usize; 6]) -> SyscallResult {
    let head = args[0];
    let len = args[1];
    let process = current_process();
    if len != core::mem::size_of::<RobustList>() {
        return Err(SyscallError::EINVAL);
    }
    let curr_id = current_task().id().as_u64();
    if process.manual_alloc_for_lazy(head.into()).is_ok() {
        let mut robust_list = process.robust_list.lock();
        robust_list.insert(curr_id, FutexRobustList::new(head, len));
        Ok(0)
    } else {
        Err(SyscallError::EINVAL)
    }
}

/// 取出对应线程的robust list
/// # Arguments
/// * pid: i32
/// * head: *mut usize
/// * len: *mut usize
pub fn syscall_get_robust_list(args: [usize; 6]) -> SyscallResult {
    let pid = args[0] as i32;
    let head = args[1] as *mut usize;
    let len = args[2] as *mut usize;

    if pid == 0 {
        let process = current_process();
        let curr_id = current_task().id().as_u64();
        if process
            .manual_alloc_for_lazy((head as usize).into())
            .is_ok()
        {
            let robust_list = process.robust_list.lock();
            if robust_list.contains_key(&curr_id) {
                let list = robust_list.get(&curr_id).unwrap();
                unsafe {
                    *head = list.head;
                    *len = list.len;
                }
            } else {
                return Err(SyscallError::EPERM);
            }
            return Ok(0);
        }
        return Err(SyscallError::EPERM);
    }
    Err(SyscallError::EPERM)
}
