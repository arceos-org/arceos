use super::ctypes::mode_t;
use crate::debug;

static mut MASK: mode_t = 0o666;

/// Set umask for open operations
///
/// Currenly only a fake implementation
#[no_mangle]
pub unsafe extern "C" fn ax_umask(mask: mode_t) -> mode_t {
    let old = MASK;
    MASK = mask & 0o777;
    debug!("umask set but not used: {}", mask);
    old
}
