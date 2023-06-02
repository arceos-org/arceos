use axerrno::LinuxResult;
use core::ffi::c_int;

/// Get current thread ID
#[no_mangle]
pub unsafe extern "C" fn ax_getpid() -> c_int {
    ax_call_body!(ax_getpid, {
        let pid = crate::thread::current().id().as_u64() as c_int;
        Ok(pid)
    })
}
