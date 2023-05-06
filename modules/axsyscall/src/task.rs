use core::time::Duration;

use axfs_os::read_file;
use axhal::time::{current_time, current_time_nanos, nanos_to_ticks};
use axprocess::{
    flags::{CloneFlags, WaitStatus},
    process::{current_process, current_task, sleep_now_task, wait_pid, yield_now_task},
    time_stat_output,
};
extern crate alloc;
use alloc::vec::Vec;
use log::info;

use crate::flags::{TimeSecs, TimeVal, UtsName, WaitFlags, TMS};
/// 处理与任务（线程）有关的系统调用

pub fn syscall_exit(exit_code: i32) -> isize {
    axlog::info!("Syscall to exit!");
    axprocess::process::exit(exit_code)
}

pub fn syscall_exec(path: *const u8, mut args: *const usize) -> isize {
    let curr_process = current_process();
    let inner = curr_process.inner.lock();
    let path = inner.memory_set.lock().translate_str(path);
    let mut args_vec = Vec::new();
    // args相当于argv，指向了参数所在的地址
    loop {
        let args_str_ptr = unsafe { *args };
        if args_str_ptr == 0 {
            break;
        }
        args_vec.push(
            inner
                .memory_set
                .lock()
                .translate_str(args_str_ptr as *const u8),
        );
        unsafe {
            args = args.add(1);
        }
    }
    drop(inner);
    let elf_data = read_file(path.as_str()).unwrap();
    let argc = args_vec.len();
    curr_process.exec(elf_data.as_slice(), args_vec);
    argc as isize
}

pub fn syscall_clone(
    flags: usize,
    user_stack: usize,
    ptid: usize,
    tls: usize,
    ctid: usize,
) -> isize {
    axlog::info!("flags: {}", flags);
    let clone_flags = CloneFlags::from_bits((flags & !0x3f) as u32).unwrap();
    let stack = if user_stack == 0 {
        None
    } else {
        Some(user_stack)
    };
    let curr_process = current_process();
    let new_task_id = curr_process.clone_task(clone_flags, stack, ptid, tls, ctid);
    new_task_id as isize
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
                        info!("Not exist!");
                        return -1;
                    }
                    WaitStatus::Running => {
                        info!("Is running!");
                        if option.contains(WaitFlags::WNOHANG) {
                            // 不予等待，直接返回0
                            return 0;
                        } else {
                            // 执行yield操作，切换任务
                            info!("wait4: yield_now_task");
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
pub fn syscall_sleep(req: *const TimeSecs, rem: *mut TimeSecs) -> isize {
    let req_time = unsafe { *req };
    let start_to_sleep = current_time();
    let dur = Duration::new(req_time.tv_sec as u64, req_time.tv_nsec as u32);
    sleep_now_task(dur);
    // 若被唤醒时时间小于请求时间，则将剩余时间写入rem
    let sleep_time = current_time() - start_to_sleep;
    if sleep_time < dur {
        let delta = (dur - sleep_time).as_nanos() as usize;
        unsafe {
            *rem = TimeSecs {
                tv_sec: delta / 1000_000_000,
                tv_nsec: delta % 1000_000_000,
            }
        };
    } else {
        unsafe {
            *rem = TimeSecs {
                tv_sec: 0,
                tv_nsec: 0,
            }
        };
    }
    0
}

/// 返回值为当前经过的时钟中断数
pub fn syscall_time(tms: *mut TMS) -> isize {
    let (_, utime_us, _, stime_us) = time_stat_output();
    unsafe {
        *tms = TMS {
            tms_utime: utime_us,
            tms_stime: stime_us,
            tms_cutime: utime_us,
            tms_cstime: stime_us,
        }
    }
    nanos_to_ticks(current_time_nanos()) as isize
}

/// 获取当前系统时间并且存储在给定结构体中
pub fn syscall_get_time_of_day(ts: *mut TimeVal) -> isize {
    let current_us = current_time_nanos() as usize / 1000;
    unsafe {
        *ts = TimeVal {
            sec: current_us / 1000_000,
            usec: current_us % 1000_000,
        }
    }
    0
}

/// 获取系统信息
pub fn syscall_uname(uts: *mut UtsName) -> isize {
    unsafe {
        *uts = UtsName::default();
    }
    0
}
