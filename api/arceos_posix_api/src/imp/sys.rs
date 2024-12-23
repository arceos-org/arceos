use core::ffi::{c_int, c_long};

use crate::ctypes;

const PAGE_SIZE_4K: usize = 4096;

/// Return system configuration infomation
///
/// Notice: currently only support what unikraft covers
pub fn sys_sysconf(name: c_int) -> c_long {
    debug!("sys_sysconf <= {}", name);

    #[cfg(feature = "alloc")]
    let (phys_pages, avail_pages) = {
        let alloc = axalloc::global_allocator();
        let avail_pages = alloc.available_pages();
        (alloc.used_pages() + avail_pages, avail_pages)
    };

    #[cfg(not(feature = "alloc"))]
    let (phys_pages, avail_pages) = {
        let mem_size = axconfig::plat::PHYS_MEMORY_SIZE;
        (mem_size / PAGE_SIZE_4K, mem_size / PAGE_SIZE_4K) // TODO
    };

    syscall_body!(sys_sysconf, {
        match name as u32 {
            // Page size
            ctypes::_SC_PAGE_SIZE => Ok(PAGE_SIZE_4K),
            // Number of processors in use
            ctypes::_SC_NPROCESSORS_ONLN => Ok(axconfig::SMP),
            // Total physical pages
            ctypes::_SC_PHYS_PAGES => Ok(phys_pages),
            // Avaliable physical pages
            ctypes::_SC_AVPHYS_PAGES => Ok(avail_pages),
            // Maximum number of files per process
            #[cfg(feature = "fd")]
            ctypes::_SC_OPEN_MAX => Ok(super::fd_ops::AX_FILE_LIMIT),
            _ => Ok(0),
        }
    })
}
