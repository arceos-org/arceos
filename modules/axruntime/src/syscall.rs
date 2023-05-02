pub mod sys_number {
    pub const SYS_WRITE: usize = 1;
    pub const SYS_EXIT: usize = 10;
    pub const SYS_SPAWN: usize = 11;
    pub const SYS_YIELD: usize = 12;
    pub const SYS_SLEEP: usize = 13;
    pub const SYS_SBRK: usize = 20;
}

use lazy_init::LazyInit;
use sys_number::*;

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
        SYS_EXIT => {
            unsafe {
                if USER_BUFFER.is_init() {
                    USER_BUFFER.get_mut_unchecked().flush();
                }
            }
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
            axtask::sleep(core::time::Duration::new(
                params[0] as u64,
                params[1] as u32,
            ));
            0
        }
        #[cfg(feature = "user-paging")]
        SYS_SBRK => {
            if let Some(value) = axmem::global_sbrk(params[0] as isize) {
                value as isize
            } else {
                -1
            }
        }

        _ => -1,
    }
}