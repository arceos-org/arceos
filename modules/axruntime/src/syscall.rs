
pub mod sys_number {
    pub const SYS_WRITE: usize = 1;
    pub const SYS_EXIT: usize = 10;
}

use sys_number::*;

pub fn syscall_handler(id: usize, params: [usize; 6]) -> isize {
    match id {
        SYS_WRITE => {
            use axhal::console::putchar;
            let print_str = unsafe {
                core::slice::from_raw_parts(params[1] as *const u8 , params[2])
            };
            for c in print_str {
                putchar(*c);
            }
            0
        },
        SYS_EXIT => {            
            axlog::info!("task exit with code {}", params[0] as isize);
            axtask::exit(0);
        }
        _ => -1,
    }
}

