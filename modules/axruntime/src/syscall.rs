pub mod sys_number {
    pub const SYS_WRITE: usize = 1;
    pub const SYS_EXIT: usize = 10;
    pub const SYS_SPAWN: usize = 11;
    pub const SYS_YIELD: usize = 12;
    pub const SYS_SLEEP: usize = 13;
}

use sys_number::*;

pub fn syscall_handler(id: usize, params: [usize; 6]) -> isize {
    trace!("syscall {}", id);
    match id {
        SYS_WRITE => {
            use axhal::console::putchar;
            if cfg!(feature = "user-paging") {
                let print_str = axmem::translate_buffer(params[1].into(), params[2]);
                for slice in &print_str {
                    for c in slice.iter() {
                        putchar(*c);
                    }
                }
                0
            } else {
                let print_str =
                    unsafe { core::slice::from_raw_parts(params[1] as *const u8, params[2]) };
                for c in print_str {
                    putchar(*c);
                }
                0
            }
        }
        SYS_EXIT => {
            axlog::info!("task exit with code {}", params[0] as isize);
            axtask::exit(0);
        }
        #[cfg(feature = "user-paging")]
        SYS_SPAWN => {
            axtask::spawn(params[0], params[1]);
            0
        }
        #[cfg(feature = "user-paging")]
        SYS_YIELD => {
            axtask::yield_now();
            0
        }
        #[cfg(feature = "user-paging")]
        SYS_SLEEP => {
            axtask::sleep(core::time::Duration::new(params[0] as u64, params[1] as u32));
            0
        }

        _ => -1,
    }
}
