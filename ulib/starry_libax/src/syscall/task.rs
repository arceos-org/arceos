use core::time::Duration;

use axconfig::{TASK_STACK_SIZE, USER_MEMORY_LIMIT};
use axhal::time::current_time;
use axprocess::{
    flags::{CloneFlags, WaitStatus},
    futex::{clear_wait, FutexRobustList},
    process::{
        current_process, current_task, set_child_tid, sleep_now_task, wait_pid, yield_now_task,
    },
};
extern crate alloc;
use super::{
    flags::{
        raw_ptr_to_ref_str, RLimit, RobustList, TimeSpec, WaitFlags, RLIMIT_AS, RLIMIT_NOFILE,
        RLIMIT_STACK,
    },
    futex::futex,
};
use alloc::{string::ToString, vec::Vec};
use axsignal::signal_no::SignalNo;
/// 处理与任务（线程）有关的系统调用

pub fn syscall_exit(exit_code: i32) -> ! {
    axprocess::process::exit(exit_code)
}

pub fn syscall_exec(path: *const u8, mut args: *const usize) -> isize {
    let curr_process = current_process();
    let inner = curr_process.inner.lock();
    let path = unsafe { raw_ptr_to_ref_str(path) }.to_string();
    let mut args_vec = Vec::new();
    // args相当于argv，指向了参数所在的地址
    loop {
        let args_str_ptr = unsafe { *args };
        if args_str_ptr == 0 {
            break;
        }
        args_vec.push(unsafe { raw_ptr_to_ref_str(args_str_ptr as *const u8) }.to_string());
        unsafe {
            args = args.add(1);
        }
    }
    drop(inner);
    // 清空futex信号列表
    clear_wait(curr_process.pid, true);
    let argc = args_vec.len();
    curr_process.exec(path, args_vec);
    argc as isize
}

pub fn syscall_clone(
    flags: usize,
    user_stack: usize,
    ptid: usize,
    tls: usize,
    ctid: usize,
) -> isize {
    let clone_flags = CloneFlags::from_bits((flags & !0x3f) as u32).unwrap();
    let signal = SignalNo::from(flags as usize & 0x3f);
    let stack = if user_stack == 0 {
        None
    } else {
        Some(user_stack)
    };
    let curr_process = current_process();
    if let Ok(new_task_id) = curr_process.clone_task(
        clone_flags,
        signal == SignalNo::SIGCHLD,
        stack,
        ptid,
        tls,
        ctid,
    ) {
        new_task_id as isize
    } else {
        -1
    }
}

/// 等待子进程完成任务，若子进程没有完成，则自身yield
/// 当前仅支持WNOHANG选项，即若未完成时则不予等待，直接返回0
pub fn syscall_wait4(pid: isize, exit_code_ptr: *mut i32, option: WaitFlags) -> isize {
    loop {
        let answer = wait_pid(pid, exit_code_ptr);
        match answer {
            Ok(pid) => {
                return pid as isize;
            }
            Err(status) => {
                match status {
                    WaitStatus::NotExist => {
                        return -1;
                    }
                    WaitStatus::Running => {
                        if option.contains(WaitFlags::WNOHANG) {
                            // 不予等待，直接返回0
                            return 0;
                        } else {
                            // 执行yield操作，切换任务
                            yield_now_task();
                        }
                    }
                    _ => {
                        panic!("Shouldn't reach here!");
                    }
                }
            }
        };
    }
}

pub fn syscall_yield() -> isize {
    yield_now_task();
    0
}

/// 当前任务进入睡眠，req指定了睡眠的时间
/// rem存储当睡眠完成时，真实睡眠时间和预期睡眠时间之间的差值
pub fn syscall_sleep(req: *const TimeSpec, rem: *mut TimeSpec) -> isize {
    let req_time = unsafe { *req };
    let start_to_sleep = current_time();
    let dur = Duration::new(req_time.tv_sec as u64, req_time.tv_nsec as u32);
    sleep_now_task(dur);
    // 若被唤醒时时间小于请求时间，则将剩余时间写入rem
    let sleep_time = current_time() - start_to_sleep;
    if rem as usize != 0 {
        if sleep_time < dur {
            let delta = (dur - sleep_time).as_nanos() as usize;
            unsafe {
                *rem = TimeSpec {
                    tv_sec: delta / 1000_000_000,
                    tv_nsec: delta % 1000_000_000,
                }
            };
        } else {
            unsafe {
                *rem = TimeSpec {
                    tv_sec: 0,
                    tv_nsec: 0,
                }
            };
        }
    }
    0
}

/// 设置tid对应的指针
/// 返回值为当前的tid
pub fn syscall_set_tid_address(tid: usize) -> isize {
    set_child_tid(tid);
    current_task().id().as_u64() as isize
}

/// 设置任务资源限制
///
/// pid 设为0时，表示应用于自己
pub fn syscall_prlimit64(
    pid: usize,
    resource: i32,
    new_limit: *const RLimit,
    old_limit: *mut RLimit,
) -> isize {
    // 当pid不为0，其实没有权利去修改其他的进程的资源限制
    let curr_process = current_process();
    if pid == 0 || pid == curr_process.pid as usize {
        match resource {
            RLIMIT_STACK => {
                if old_limit as usize != 0 {
                    unsafe {
                        *old_limit = RLimit {
                            rlim_cur: TASK_STACK_SIZE as u64,
                            rlim_max: TASK_STACK_SIZE as u64,
                        };
                    }
                }
            }
            RLIMIT_NOFILE => {
                // 仅支持修改最大文件数
                let mut inner = curr_process.inner.lock();
                if old_limit as usize != 0 {
                    let limit = inner.fd_manager.limit;
                    unsafe {
                        *old_limit = RLimit {
                            rlim_cur: limit as u64,
                            rlim_max: limit as u64,
                        };
                    }
                }
                if new_limit as usize != 0 {
                    let new_limit = unsafe { (*new_limit).rlim_cur };
                    inner.fd_manager.limit = new_limit as usize;
                }
            }
            RLIMIT_AS => {
                if old_limit as usize != 0 {
                    unsafe {
                        *old_limit = RLimit {
                            rlim_cur: USER_MEMORY_LIMIT as u64,
                            rlim_max: USER_MEMORY_LIMIT as u64,
                        };
                    }
                }
            }
            _ => {}
        }
    }
    0
}

/// 当前不涉及多核情况
pub fn syscall_getpid() -> isize {
    let curr = current_task();
    let pid = curr.get_process_id();
    pid as isize
}

pub fn syscall_getppid() -> isize {
    let curr_process = current_process();
    let inner = curr_process.inner.lock();
    let parent_id = inner.parent;
    drop(inner);
    parent_id as isize
}

pub fn syscall_umask(new_mask: i32) -> isize {
    let curr_process = current_process();
    let mut inner = curr_process.inner.lock();
    inner.fd_manager.set_mask(new_mask) as isize
}

/// 获取用户 id。在实现多用户权限前默认为最高权限
pub fn syscall_getuid() -> isize {
    0
}

/// 获取有效用户 id，即相当于哪个用户的权限。在实现多用户权限前默认为最高权限
pub fn syscall_geteuid() -> isize {
    0
}

/// 获取用户组 id。在实现多用户权限前默认为最高权限
pub fn syscall_getgid() -> isize {
    0
}

/// 获取有效用户组 id，即相当于哪个用户的权限。在实现多用户权限前默认为最高权限
pub fn syscall_getegid() -> isize {
    0
}

pub fn syscall_gettid() -> isize {
    current_task().id().as_u64() as isize
}

pub fn syscall_futex(
    vaddr: usize,
    futex_op: i32,
    futex_val: u32,
    time_out_val: usize,
    vaddr2: usize,
    val3: u32,
) -> isize {
    let process = current_process();
    let inner = process.inner.lock();
    let timeout = if time_out_val != 0
        && inner
            .memory_set
            .lock()
            .manual_alloc_for_lazy(time_out_val.into())
            .is_ok()
    {
        let time_sepc: TimeSpec = unsafe { *(time_out_val as *const TimeSpec) };
        time_sepc.to_nano()
    } else {
        // usize::MAX
        0
    };
    // 释放锁，防止任务无法被调度
    drop(inner);
    match futex(
        vaddr.into(),
        futex_op,
        futex_val,
        timeout,
        vaddr2.into(),
        val3,
    ) {
        Ok(ans) => ans as isize,
        Err(errno) => errno as isize,
    }
}

/// 内核只发挥存储的作用
/// 但要保证head对应的地址已经被分配
pub fn syscall_set_robust_list(head: usize, len: usize) -> isize {
    let process = current_process();
    let mut inner = process.inner.lock();
    if len != core::mem::size_of::<RobustList>() {
        return -1;
    }
    let curr_id = current_task().id().as_u64();
    if inner
        .memory_set
        .lock()
        .manual_alloc_for_lazy(head.into())
        .is_ok()
    {
        if inner.robust_list.contains_key(&curr_id) {
            let list = inner.robust_list.get_mut(&curr_id).unwrap();
            list.head = head;
            list.len = len;
        } else {
            inner
                .robust_list
                .insert(curr_id, FutexRobustList::new(head, len));
        }
        0
    } else {
        -1
    }
}

/// 取出对应线程的robust list
pub fn syscall_get_robust_list(pid: i32, head: *mut usize, len: *mut usize) -> isize {
    if pid == 0 {
        let process = current_process();
        let inner = process.inner.lock();
        let curr_id = current_task().id().as_u64();
        if inner
            .memory_set
            .lock()
            .manual_alloc_for_lazy((head as usize).into())
            .is_ok()
        {
            if inner.robust_list.contains_key(&curr_id) {
                let list = inner.robust_list.get(&curr_id).unwrap();
                unsafe {
                    *head = list.head;
                    *len = list.len;
                }
            } else {
                return -1;
            }
            return 0;
        } else {
            return -1;
        }
    }
    -1
}
