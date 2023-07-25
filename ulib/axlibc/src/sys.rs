use super::ctypes;
use core::ffi::{c_int, c_long};

const PAGE_SIZE_4K: usize = 4096;

/// Return system configuration infomation
///
/// Notice: currently only support what unikraft covers
#[no_mangle]
pub unsafe extern "C" fn ax_sysconf(name: c_int) -> c_long {
    debug!("ax_sysconf <= {}", name);
    ax_call_body!(ax_sysconf, {
        match name as u32 {
            // Page size
            ctypes::_SC_PAGE_SIZE => Ok(PAGE_SIZE_4K as c_long),
            // Total physical pages
            ctypes::_SC_PHYS_PAGES => Ok((axconfig::PHYS_MEMORY_SIZE / PAGE_SIZE_4K) as c_long),
            // Number of processors in use
            ctypes::_SC_NPROCESSORS_ONLN => Ok(axconfig::SMP as c_long),
            // Avaliable physical pages
            #[cfg(feature = "alloc")]
            ctypes::_SC_AVPHYS_PAGES => Ok(axalloc::global_allocator().available_pages() as c_long),
            // Maximum number of files per process
            #[cfg(feature = "fd")]
            ctypes::_SC_OPEN_MAX => Ok(super::fd_ops::AX_FILE_LIMIT as c_long),
            _ => Ok(0),
        }
    })
}
