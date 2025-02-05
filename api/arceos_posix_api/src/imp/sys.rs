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
            // Number of processors in use
            ctypes::_SC_NPROCESSORS_ONLN => Ok(axconfig::SMP),
            // Total physical pages
            ctypes::_SC_PHYS_PAGES => Ok(axhal::mem::total_ram_size() / PAGE_SIZE_4K),
            // Avaliable physical pages
            ctypes::_SC_AVPHYS_PAGES => {
                #[cfg(feature = "alloc")]
                {
                    Ok(axalloc::global_allocator().available_pages())
                }
                #[cfg(not(feature = "alloc"))]
                {
                    let total_pages = axhal::mem::total_ram_size() / PAGE_SIZE_4K;
                    let reserved_pages = axhal::mem::reserved_phys_ram_ranges()
                        .iter()
                        .map(|range| range.1 / PAGE_SIZE_4K)
                        .sum::<usize>();
                    Ok(total_pages - reserved_pages)
                }
            }
            // Maximum number of files per process
            #[cfg(feature = "fd")]
            ctypes::_SC_OPEN_MAX => Ok(super::fd_ops::AX_FILE_LIMIT),
            _ => Ok(0),
        }
    })
}
