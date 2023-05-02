#![cfg_attr(not(test), no_std)]
use task::syscall_clone;

use self::{
    fs::syscall_write,
    task::{syscall_exec, syscall_exit},
};
extern crate log;
extern crate axlog;

extern crate alloc;
mod fs;
mod task;
pub const SYSCALL_WRITE: usize = 64;
pub const SYSCALL_EXIT: usize = 93;
pub const SYSCALL_CLONE: usize = 220;
pub const SYSCALL_EXEC: usize = 221;

#[no_mangle]
// #[cfg(feature = "user")]
pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => syscall_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => syscall_exit(),
        SYSCALL_EXEC => syscall_exec(args[0] as *const u8, args[1] as *const usize),
        SYSCALL_CLONE => syscall_clone(args[0], args[1], args[2], args[3], args[4]),
        _ => {
            panic!("Invalid Syscall Id: {}!", syscall_id);
        }
    }
}
