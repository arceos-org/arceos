use core::ffi::{c_int, c_long};

use crate::ctypes;

pub const PAGE_SIZE_4K: usize = 4096;

/// Return sysinfo struct
#[no_mangle]
pub unsafe extern "C" fn sys_sysinfo(info: *mut ctypes::sysinfo) -> c_int {
    debug!("sys_sysinfo");
    syscall_body!(ax_sysinfo, {
        let info_mut = info.as_mut().unwrap();

        // If the kernel booted less than 1 second, it will be 0.
        info_mut.uptime = axhal::time::current_time().as_secs() as c_long;

        info_mut.loads = [0; 3];
        #[cfg(feature = "axtask")]
        {
            axtask::loadavg::get_avenrun(&mut info_mut.loads);
        }

        info_mut.sharedram = 0;
        // TODO
        info_mut.bufferram = 0;

        info_mut.totalram = 0;
        info_mut.freeram = 0;
        #[cfg(feature = "alloc")]
        {
            use core::ffi::c_ulong;
            let allocator = axalloc::global_allocator();
            info_mut.freeram = (allocator.available_bytes()
                + allocator.available_pages() * PAGE_SIZE_4K)
                as c_ulong;
            info_mut.totalram = info_mut.freeram + allocator.used_bytes() as c_ulong;
        }

        // TODO
        info_mut.totalswap = 0;
        info_mut.freeswap = 0;

        info_mut.procs = 1;

        // unused in 64-bit
        info_mut.totalhigh = 0;
        info_mut.freehigh = 0;

        info_mut.mem_unit = 1;

        Ok(0)
    })
}
