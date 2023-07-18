use axhal::time::{current_time_nanos, nanos_to_ticks};
use axprocess::{process::current_process, time_stat_output};

use super::flags::{TimeSpec, TimeVal, UtsName, TMS};

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

/// 用于获取当前系统时间并且存储在对应的结构体中
pub fn syscall_clock_get_time(_clock_id: usize, ts: *mut TimeSpec) -> isize {
    let process = current_process();
    let inner = process.inner.lock();
    if inner
        .memory_set
        .lock()
        .manual_alloc_type_for_lazy(ts as *const TimeSpec)
        .is_err()
    {
        return -1;
    }
    unsafe {
        (*ts) = TimeSpec::now();
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
