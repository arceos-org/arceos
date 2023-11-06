use core::{slice::from_raw_parts_mut, time::Duration};

use axhal::time::{current_time, current_time_nanos, nanos_to_ticks, NANOS_PER_SEC};

use axprocess::{current_process, current_task, time_stat_output};
use rand::{rngs::SmallRng, Fill, SeedableRng};

use syscall_utils::{
    ClockId, ITimerVal, RusageFlags, SysInfo, SyscallError, SyscallResult, TimeSecs, TimeVal,
    UtsName, TMS,
};

/// 返回值为当前经过的时钟中断数
pub fn syscall_time(tms: *mut TMS) -> SyscallResult {
    let (_, utime_us, _, stime_us) = time_stat_output();
    unsafe {
        *tms = TMS {
            tms_utime: utime_us,
            tms_stime: stime_us,
            tms_cutime: utime_us,
            tms_cstime: stime_us,
        }
    }
    Ok(nanos_to_ticks(current_time_nanos()) as isize)
}

/// 获取当前系统时间并且存储在给定结构体中
pub fn syscall_get_time_of_day(ts: *mut TimeVal) -> SyscallResult {
    let current_us = current_time_nanos() as usize / 1000;
    unsafe {
        *ts = TimeVal {
            sec: current_us / 1000_000,
            usec: current_us % 1000_000,
        }
    }
    Ok(0)
}

/// 用于获取当前系统时间并且存储在对应的结构体中
pub fn syscall_clock_get_time(_clock_id: usize, ts: *mut TimeSecs) -> SyscallResult {
    unsafe {
        (*ts) = TimeSecs::now();
    }
    Ok(0)
}

/// 获取系统信息
pub fn syscall_uname(uts: *mut UtsName) -> SyscallResult {
    unsafe {
        *uts = UtsName::default();
    }
    Ok(0)
}

/// 获取系统的启动时间和内存信息，当前仅支持启动时间
pub fn syscall_sysinfo(info: *mut SysInfo) -> SyscallResult {
    let process = current_process();
    if process
        .manual_alloc_type_for_lazy(info as *const SysInfo)
        .is_err()
    {
        return Err(SyscallError::EFAULT);
    }

    unsafe {
        // 获取以秒为单位的时间
        (*info).uptime = (current_time_nanos() / NANOS_PER_SEC) as isize;
    }
    Ok(0)
}

pub fn syscall_settimer(
    which: usize,
    new_value: *const ITimerVal,
    old_value: *mut ITimerVal,
) -> SyscallResult {
    let process = current_process();

    if new_value.is_null() {
        return Err(SyscallError::EFAULT);
    }

    let new_value = match process.manual_alloc_type_for_lazy(new_value) {
        Ok(_) => unsafe { &*new_value },
        Err(_) => return Err(SyscallError::EFAULT),
    };

    if !old_value.is_null() {
        if process.manual_alloc_type_for_lazy(old_value).is_err() {
            return Err(SyscallError::EFAULT);
        }

        let (time_interval_us, time_remained_us) = current_task().timer_output();
        unsafe {
            (*old_value).it_interval = TimeVal::from_micro(time_interval_us);
            (*old_value).it_value = TimeVal::from_micro(time_remained_us);
        }
    }
    let (time_interval_ns, time_remained_ns) = (
        new_value.it_interval.to_nanos(),
        new_value.it_value.to_nanos(),
    );
    if current_task().set_timer(time_interval_ns, time_remained_ns, which) {
        Ok(0)
    } else {
        // 说明which参数错误
        Err(SyscallError::EFAULT)
    }
}

pub fn syscall_gettimer(_which: usize, value: *mut ITimerVal) -> SyscallResult {
    let process = current_process();
    if process
        .manual_alloc_type_for_lazy(value as *const ITimerVal)
        .is_err()
    {
        return Err(SyscallError::EFAULT);
    }
    let (time_interval_us, time_remained_us) = current_task().timer_output();
    unsafe {
        (*value).it_interval = TimeVal::from_micro(time_interval_us);
        (*value).it_value = TimeVal::from_micro(time_remained_us);
    }
    Ok(0)
}

pub fn syscall_getrusage(who: i32, utime: *mut TimeVal) -> SyscallResult {
    let stime: *mut TimeVal = unsafe { utime.add(1) };
    let process = current_process();
    if process.manual_alloc_type_for_lazy(utime).is_err()
        || process.manual_alloc_type_for_lazy(stime).is_err()
    {
        return Err(SyscallError::EFAULT);
    }
    if RusageFlags::from(who).is_some() {
        let (_, utime_us, _, stime_us) = time_stat_output();
        unsafe {
            *utime = TimeVal::from_micro(utime_us);
            *stime = TimeVal::from_micro(stime_us);
        }
        Ok(0)
    } else {
        Err(SyscallError::EINVAL)
    }
}

pub fn syscall_getrandom(buf: *mut u8, len: usize, _flags: usize) -> SyscallResult {
    let process = current_process();

    if process
        .manual_alloc_range_for_lazy(
            (buf as usize).into(),
            unsafe { buf.add(len) as usize }.into(),
        )
        .is_err()
    {
        return Err(SyscallError::EFAULT);
    }

    let buf = unsafe { from_raw_parts_mut(buf, len) };

    // TODO: flags
    // - GRND_RANDOM: use /dev/random or /dev/urandom
    // - GRND_NONBLOCK: EAGAIN when block
    let mut rng = SmallRng::from_seed([0; 32]);
    buf.try_fill(&mut rng).unwrap();

    Ok(buf.len() as isize)
}

/// # 获取时钟精度
///
/// ## args：
/// * id：时钟种类，当前仅支持CLOCK_MONOTONIC
///
/// * res：存储时钟精度的结构体的地址
pub fn syscall_clock_getres(id: usize, res: *mut TimeSecs) -> SyscallResult {
    let id = if let Ok(opt) = ClockId::try_from(id) {
        opt
    } else {
        return Err(SyscallError::EINVAL);
    };

    if id != ClockId::CLOCK_MONOTONIC {
        // 暂时不支持其他类型
        return Err(SyscallError::EINVAL);
    }

    let process = current_process();
    if process.manual_alloc_type_for_lazy(res).is_err() {
        return Err(SyscallError::EFAULT);
    }

    unsafe {
        (*res) = TimeSecs {
            tv_nsec: 1,
            tv_sec: 0,
        };
    }

    Ok(0)
}

/// # 指定任务进行睡眠
///
/// ## args:
/// * id: 指定使用的时钟ID，对应结构体为ClockId
///
/// * flags：指定是使用相对时间还是绝对时间
///
/// * request：指定睡眠的时间，根据flags划分为相对时间或者绝对时间
///
/// * remain：存储剩余睡眠时间。当任务提前醒来时，如果flags不为绝对时间，且remain不为空，则将剩余存储时间存进remain所指向地址。
///
/// 若睡眠被信号处理打断或者遇到未知错误，则返回对应错误码
pub fn syscall_clock_nanosleep(
    id: usize,
    flags: usize,
    request: *const TimeSecs,
    remain: *mut TimeSecs,
) -> SyscallResult {
    const TIMER_ABSTIME: usize = 1;
    let id = if let Ok(opt) = ClockId::try_from(id) {
        opt
    } else {
        return Err(SyscallError::EINVAL);
    };

    if id != ClockId::CLOCK_MONOTONIC {
        // 暂时不支持其他类型
        return Err(SyscallError::EINVAL);
    }

    let process = current_process();

    if process.manual_alloc_type_for_lazy(request).is_err() {
        return Err(SyscallError::EFAULT);
    }
    let request_time = unsafe { *request };
    let request_time = Duration::new(request_time.tv_sec as u64, request_time.tv_nsec as u32);
    let deadline = if flags != TIMER_ABSTIME {
        current_time() + request_time
    } else {
        if request_time < current_time() {
            return Ok(0);
        }
        request_time
    };

    axtask::sleep_until(deadline);

    let current_time = current_time();
    if current_time < deadline && !remain.is_null() {
        if process.manual_alloc_type_for_lazy(remain).is_err() {
            return Err(SyscallError::EFAULT);
        } else {
            let delta = (deadline - current_time).as_nanos() as usize;
            unsafe {
                *remain = TimeSecs {
                    tv_sec: delta / 1000_000_000,
                    tv_nsec: delta % 1000_000_000,
                }
            };
            return Err(SyscallError::EINTR);
        }
    }
    Ok(0)
}
