use arceos_posix_api::{config, ctypes, PAGE_SIZE_4K};
use core::ffi::{c_int, c_long};

/// Return system configuration infomation
///
/// Notice: currently only support what unikraft covers
#[no_mangle]
pub unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    match name as u32 {
        // Page size
        ctypes::_SC_PAGE_SIZE => PAGE_SIZE_4K as c_long,
        // Total physical pages
        ctypes::_SC_PHYS_PAGES => (config::PHYS_MEMORY_SIZE / PAGE_SIZE_4K) as c_long,
        // Number of processors in use
        ctypes::_SC_NPROCESSORS_ONLN => config::SMP as c_long,
        // Avaliable physical pages
        #[cfg(feature = "axalloc")]
        ctypes::_SC_AVPHYS_PAGES => axalloc::global_allocator().available_pages(),
        // Maximum number of files per process
        #[cfg(feature = "fd")]
        ctypes::_SC_OPEN_MAX => super::fd_ops::AX_FILE_LIMIT,
        _ => 0,
    }
}
