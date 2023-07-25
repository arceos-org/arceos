use axhal::time::{current_time_nanos, nanos_to_ticks, NANOS_PER_SEC};
use axprocess::{
    process::{current_process, current_task},
    time_stat_output,
};

use super::{
    flags::{ITimerVal, RusageFlags, SysInfo, TimeSecs, TimeVal, UtsName, TMS},
    syscall_id::ErrorNo,
};

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
pub fn syscall_clock_get_time(_clock_id: usize, ts: *mut TimeSecs) -> isize {
    unsafe {
        (*ts) = TimeSecs::now();
        // info!("ts: {:?}", *ts);
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

/// 获取系统的启动时间和内存信息，当前仅支持启动时间
pub fn syscall_sysinfo(info: *mut SysInfo) -> isize {
    let process = current_process();
    let inner = process.inner.lock();
    if inner
        .memory_set
        .lock()
        .manual_alloc_type_for_lazy(info as *const SysInfo)
        .is_err()
    {
        return ErrorNo::EFAULT as isize;
    }

    unsafe {
        // 获取以秒为单位的时间
        (*info).uptime = (current_time_nanos() / NANOS_PER_SEC) as isize;
    }
    0
}

pub fn syscall_settimer(
    which: usize,
    new_value: *const ITimerVal,
    old_value: *mut ITimerVal,
) -> isize {
    let process = current_process();
    let inner = process.inner.lock();
    if inner
        .memory_set
        .lock()
        .manual_alloc_type_for_lazy(new_value as *const TimeVal)
        .is_err()
    {
        return ErrorNo::EFAULT as isize;
    }
    if old_value as usize != 0 {
        if inner
            .memory_set
            .lock()
            .manual_alloc_type_for_lazy(old_value as *const TimeVal)
            .is_err()
        {
            return ErrorNo::EFAULT as isize;
        }
        let (time_interval_us, time_remained_us) = current_task().timer_output();
        unsafe {
            (*old_value).it_interval = TimeVal::from_micro(time_interval_us);
            (*old_value).it_value = TimeVal::from_micro(time_remained_us);
        }
    }
    let (time_interval_ns, time_remained_ns) = unsafe {
        (
            (*new_value).it_interval.to_nanos(),
            (*new_value).it_value.to_nanos(),
        )
    };
    if current_task().set_timer(time_interval_ns, time_remained_ns, which) {
        0
    } else {
        // 说明which参数错误
        ErrorNo::EFAULT as isize
    }
}

pub fn syscall_gettimer(_which: usize, value: *mut ITimerVal) -> isize {
    let process = current_process();
    let inner = process.inner.lock();
    if inner
        .memory_set
        .lock()
        .manual_alloc_type_for_lazy(value as *const ITimerVal)
        .is_err()
    {
        return ErrorNo::EFAULT as isize;
    }
    let (time_interval_us, time_remained_us) = current_task().timer_output();
    unsafe {
        (*value).it_interval = TimeVal::from_micro(time_interval_us);
        (*value).it_value = TimeVal::from_micro(time_remained_us);
    }
    0
}

pub fn syscall_getrusage(who: i32, utime: *mut TimeVal) -> isize {
    let stime: *mut TimeVal = unsafe { utime.add(1) };
    let process = current_process();
    let inner = process.inner.lock();
    let mut memory_set = inner.memory_set.lock();
    if memory_set.manual_alloc_type_for_lazy(utime).is_err()
        || memory_set.manual_alloc_type_for_lazy(stime).is_err()
    {
        return ErrorNo::EFAULT as isize;
    }
    if RusageFlags::from(who).is_some() {
        let (_, utime_us, _, stime_us) = time_stat_output();
        unsafe {
            *utime = TimeVal::from_micro(utime_us);
            *stime = TimeVal::from_micro(stime_us);
        }
        0
    } else {
        ErrorNo::EINVAL as isize
    }
}
