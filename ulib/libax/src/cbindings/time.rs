use axerrno::{LinuxError, LinuxResult};
use axhal::time::{current_time, NANOS_PER_SEC};
use core::ffi::{c_int, c_long};
use core::time::Duration;

use super::ctypes;

/// Get clock time since booting
#[no_mangle]
pub unsafe extern "C" fn ax_clock_gettime(ts: *mut ctypes::timespec) -> c_int {
    ax_call_body!(ax_clock_gettime, {
        if ts.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let now = current_time();
        let ret = ctypes::timespec {
            tv_sec: now.as_secs() as c_long,
            tv_nsec: now.subsec_nanos() as c_long,
        };
        unsafe {
            *ts = ret;
        }
        debug!("ax_clock_gettime: {}.{:09}s", ret.tv_sec, ret.tv_nsec);
        Ok(0)
    })
}

/// Sleep some nanoseconds
///
/// TODO: should be woken by signals, and set errno
#[no_mangle]
pub unsafe extern "C" fn ax_nanosleep(
    req: *const ctypes::timespec,
    rem: *mut ctypes::timespec,
) -> c_int {
    ax_call_body!(ax_nanosleep, {
        if req.is_null() || (*req).tv_nsec < 0 || (*req).tv_nsec > 999999999 {
            return Err(LinuxError::EINVAL);
        }

        debug!("ax_nanosleep <= {}.{:09}s", (*req).tv_sec, (*req).tv_nsec);
        let total_nano = (*req).tv_sec as u64 * NANOS_PER_SEC + (*req).tv_nsec as u64;
        let before = current_time().as_nanos() as u64;

        crate::thread::sleep(Duration::from_nanos(total_nano));

        let after = current_time().as_nanos() as u64;
        let diff = after - before;

        if diff < total_nano {
            if !rem.is_null() {
                (*rem).tv_sec = ((total_nano - diff) / NANOS_PER_SEC) as i64;
                (*rem).tv_nsec = ((total_nano - diff) % NANOS_PER_SEC) as i64;
            }
            return Err(LinuxError::EINTR);
        }
        Ok(0)
    })
}
