use axerrno::LinuxError;
use core::ffi::{c_int, c_long};
use core::time::Duration;

use crate::ctypes;
use crate::ctypes::{CLOCK_MONOTONIC, CLOCK_REALTIME};

impl From<ctypes::timespec> for Duration {
    fn from(ts: ctypes::timespec) -> Self {
        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
    }
}

impl From<ctypes::timeval> for Duration {
    fn from(tv: ctypes::timeval) -> Self {
        Duration::new(tv.tv_sec as u64, tv.tv_usec as u32 * 1000)
    }
}

impl From<Duration> for ctypes::timespec {
    fn from(d: Duration) -> Self {
        ctypes::timespec {
            tv_sec: d.as_secs() as c_long,
            tv_nsec: d.subsec_nanos() as c_long,
        }
    }
}

impl From<Duration> for ctypes::timeval {
    fn from(d: Duration) -> Self {
        ctypes::timeval {
            tv_sec: d.as_secs() as c_long,
            tv_usec: d.subsec_micros() as c_long,
        }
    }
}

/// Get clock time since booting
pub unsafe fn sys_clock_gettime(clk: ctypes::clockid_t, ts: *mut ctypes::timespec) -> c_int {
    syscall_body!(sys_clock_gettime, {
        if ts.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let now = match clk as u32 {
            CLOCK_REALTIME => axhal::time::wall_time().into(),
            CLOCK_MONOTONIC => axhal::time::monotonic_time().into(),
            _ => {
                warn!("Called sys_clock_gettime for unsupported clock {}", clk);
                return Err(LinuxError::EINVAL);
            }
        };
        unsafe { *ts = now };
        debug!("sys_clock_gettime: {}.{:09}s", now.tv_sec, now.tv_nsec);
        Ok(0)
    })
}

/// Sleep some nanoseconds
///
/// TODO: should be woken by signals, and set errno
pub unsafe fn sys_nanosleep(req: *const ctypes::timespec, rem: *mut ctypes::timespec) -> c_int {
    syscall_body!(sys_nanosleep, {
        unsafe {
            if req.is_null() || (*req).tv_nsec < 0 || (*req).tv_nsec > 999999999 {
                return Err(LinuxError::EINVAL);
            }
        }

        let dur = unsafe {
            debug!("sys_nanosleep <= {}.{:09}s", (*req).tv_sec, (*req).tv_nsec);
            Duration::from(*req)
        };

        let now = axhal::time::monotonic_time();

        #[cfg(feature = "multitask")]
        axtask::sleep(dur);
        #[cfg(not(feature = "multitask"))]
        axhal::time::busy_wait(dur);

        let after = axhal::time::monotonic_time();
        let actual = after - now;

        if let Some(diff) = dur.checked_sub(actual) {
            if !rem.is_null() {
                unsafe { (*rem) = diff.into() };
            }
            return Err(LinuxError::EINTR);
        }
        Ok(0)
    })
}

/// Get current system time and store in specific struct
pub unsafe fn sys_get_time_of_day(ts: *mut ctypes::timeval) -> c_int {
    syscall_body!(sys_get_time_of_day, {
        let current_us = axhal::time::monotonic_time_nanos() as usize / 1000;
        unsafe {
            *ts = ctypes::timeval {
                tv_sec: (current_us / 1_000_000) as i64,
                tv_usec: (current_us % 1_000_000) as i64,
            }
        }
        Ok(0)
    })
}
