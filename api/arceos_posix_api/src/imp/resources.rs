use crate::ctypes;
use axerrno::LinuxError;
use core::ffi::c_int;

/// Get resource limitations
///
/// TODO: support more resource types
pub unsafe fn sys_getrlimit(resource: c_int, rlimits: *mut ctypes::rlimit) -> c_int {
    debug!("sys_getrlimit <= {} {:#x}", resource, rlimits as usize);
    syscall_body!(sys_getrlimit, {
        match resource as u32 {
            ctypes::RLIMIT_DATA => {}
            ctypes::RLIMIT_STACK => {}
            ctypes::RLIMIT_NOFILE => {}
            _ => return Err(LinuxError::EINVAL),
        }
        if rlimits.is_null() {
            return Ok(0);
        }
        match resource as u32 {
            ctypes::RLIMIT_STACK => unsafe {
                (*rlimits).rlim_cur = axconfig::TASK_STACK_SIZE as _;
                (*rlimits).rlim_max = axconfig::TASK_STACK_SIZE as _;
            },
            #[cfg(feature = "fd")]
            ctypes::RLIMIT_NOFILE => unsafe {
                (*rlimits).rlim_cur = super::fd_ops::AX_FILE_LIMIT as _;
                (*rlimits).rlim_max = super::fd_ops::AX_FILE_LIMIT as _;
            },
            _ => {}
        }
        Ok(0)
    })
}

/// Set resource limitations
///
/// TODO: support more resource types
pub unsafe fn sys_setrlimit(resource: c_int, rlimits: *mut crate::ctypes::rlimit) -> c_int {
    debug!("sys_setrlimit <= {} {:#x}", resource, rlimits as usize);
    syscall_body!(sys_setrlimit, {
        match resource as u32 {
            crate::ctypes::RLIMIT_DATA => {}
            crate::ctypes::RLIMIT_STACK => {}
            crate::ctypes::RLIMIT_NOFILE => {}
            _ => return Err(LinuxError::EINVAL),
        }
        // Currently do not support set resources
        Ok(0)
    })
}
