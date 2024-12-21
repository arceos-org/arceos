use crate::{ctypes, utils::e};
use arceos_posix_api as api;
use core::ffi::{c_int, c_void};

/// Returns the `pthread` struct of current thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_self() -> ctypes::pthread_t {
    api::sys_pthread_self()
}

/// Create a new thread with the given entry point and argument.
///
/// If successful, it stores the pointer to the newly created `struct __pthread`
/// in `res` and returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_create(
    res: *mut ctypes::pthread_t,
    attr: *const ctypes::pthread_attr_t,
    start_routine: extern "C" fn(arg: *mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    e(api::sys_pthread_create(res, attr, start_routine, arg))
}

/// Exits the current thread. The value `retval` will be returned to the joiner.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_exit(retval: *mut c_void) -> ! {
    api::sys_pthread_exit(retval)
}

/// Waits for the given thread to exit, and stores the return value in `retval`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_join(
    thread: ctypes::pthread_t,
    retval: *mut *mut c_void,
) -> c_int {
    e(api::sys_pthread_join(thread, retval))
}

/// Initialize a mutex.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_init(
    mutex: *mut ctypes::pthread_mutex_t,
    attr: *const ctypes::pthread_mutexattr_t,
) -> c_int {
    e(api::sys_pthread_mutex_init(mutex, attr))
}

/// Lock the given mutex.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_lock(mutex: *mut ctypes::pthread_mutex_t) -> c_int {
    e(api::sys_pthread_mutex_lock(mutex))
}

/// Unlock the given mutex.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut ctypes::pthread_mutex_t) -> c_int {
    e(api::sys_pthread_mutex_unlock(mutex))
}
