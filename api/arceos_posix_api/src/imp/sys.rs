use core::ffi::{c_int, c_long};

use crate::ctypes;

const PAGE_SIZE_4K: usize = 4096;

/// Return system configuration infomation
///
/// Notice: currently only support what unikraft covers
pub fn sys_sysconf(name: c_int) -> c_long {
    debug!("sys_sysconf <= {}", name);
    syscall_body!(sys_sysconf, {
        match name as u32 {
            // Page size
            ctypes::_SC_PAGE_SIZE => Ok(PAGE_SIZE_4K),
            // Total physical pages
            ctypes::_SC_PHYS_PAGES => Ok(axconfig::PHYS_MEMORY_SIZE / PAGE_SIZE_4K),
            // Number of processors in use
            ctypes::_SC_NPROCESSORS_ONLN => Ok(axconfig::SMP),
            // Avaliable physical pages
            #[cfg(feature = "alloc")]
            ctypes::_SC_AVPHYS_PAGES => Ok(axalloc::global_allocator().available_pages()),
            // Maximum number of files per process
            #[cfg(feature = "fd")]
            ctypes::_SC_OPEN_MAX => Ok(super::fd_ops::AX_FILE_LIMIT),
            _ => Ok(0),
        }
    })
}
