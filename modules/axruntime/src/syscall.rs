use axmem::copy_data_to_user;
use lazy_init::LazyInit;
use syscall_number::*;

extern crate alloc;
struct UserBuffer {
    data: alloc::vec::Vec<u8>,
}
impl UserBuffer {
    fn flush(&mut self) {
        use axhal::console::putchar;
        info!("Writing user content");
        for i in &self.data {
            putchar(*i);
        }
        self.data.clear();
    }
    #[allow(dead_code)]
    fn putchar(&mut self, c: u8) {
        self.data.push(c);
        if c == b'\n' {
            self.flush();
        }
    }
}
static mut USER_BUFFER: LazyInit<UserBuffer> = LazyInit::new();

pub fn syscall_handler(id: usize, params: [usize; 6]) -> isize {
    trace!("syscall {}", id);
    match id {
        #[cfg(not(feature = "scheme"))]
        SYS_WRITE => {
            unsafe {
                if !USER_BUFFER.is_init() {
                    USER_BUFFER.init_by(UserBuffer {
                        data: alloc::vec![],
                    })
                }
            }
            use axhal::console::putchar;
            if cfg!(feature = "user-paging") {
                let print_str = axmem::translate_buffer(params[1].into(), params[2]);
                for slice in &print_str {
                    for c in slice.iter() {
                        unsafe {
                            USER_BUFFER.get_mut_unchecked().putchar(*c);
                        }
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
        #[cfg(feature = "scheme")]
        file_syscall if file_syscall & SYS_CLASS != 0 => axscheme::syscall_handler(id, params),
        SYS_EXIT => {
            unsafe {
                if USER_BUFFER.is_init() {
                    USER_BUFFER.get_mut_unchecked().flush();
                }
            }
            axlog::info!("task exit with code {}", params[0] as isize);

            #[cfg(feature = "process")]
            axprocess::exit_current(params[0] as i32);

            axtask::exit(params[0] as i32);
        }
        #[cfg(feature = "user-paging")]
        SYS_SPAWN => {
            #[allow(unused_variables)]
            let task = axtask::spawn_args(params[0], params[1]);

            #[cfg(feature = "process")]
            axprocess::add_task(task);

            0
        }
        #[cfg(feature = "user-paging")]
        SYS_YIELD => {
            axtask::yield_now();
            0
        }
        #[cfg(feature = "user-paging")]
        SYS_SLEEP => {
            axtask::sleep(core::time::Duration::new(
                params[0] as u64,
                params[1] as u32,
            ));
            0
        }
        SYS_TIME_NANO => axhal::time::current_time_nanos() as isize,
        #[cfg(feature = "user-paging")]
        SYS_SBRK => {
            if let Some(value) = axmem::global_sbrk(params[0] as isize) {
                value as isize
            } else {
                -1
            }
        }
        #[cfg(feature = "futex")]
        SYS_FUTEX => {
            if let Some(phy_addr) = axmem::translate_addr(params[0].into()) {
                axsync::futex::futex_call(phy_addr.into(), params[1], params[2] as u32)
            } else {
                -1
            }
        }
        #[cfg(feature = "process")]
        SYS_FORK => axprocess::fork() as isize,

        #[cfg(feature = "process")]
        SYS_WAIT => {
            let (id, code) = axprocess::wait(params[0] as u64);
            copy_data_to_user(0, params[1] as *const u8, &code);
            id as isize
        }
        _ => -1,
    }
}
