use core::ffi::c_int;

pub fn e(ret: c_int) -> c_int {
    if ret < 0 {
        crate::errno::set_errno(ret.abs());
        -1
    } else {
        ret as _
    }
}
