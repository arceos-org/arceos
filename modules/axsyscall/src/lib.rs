#![cfg_attr(not(test), no_std)]
use flags::WaitFlags;
use log::info;
use task::{syscall_clone, syscall_getpid, syscall_getppid, syscall_wait4};

use self::{
    fs::syscall_write,
    task::{syscall_exec, syscall_exit},
};
extern crate axlog;
extern crate log;

extern crate alloc;
mod flags;
mod fs;
mod mem;
mod task;
pub const SYSCALL_WRITE: usize = 64;
pub const SYSCALL_EXIT: usize = 93;
pub const SYSCALL_GETPID: usize = 172;
pub const SYSCALL_GETPPID: usize = 173;
pub const SYSCALL_CLONE: usize = 220;
pub const SYSCALL_EXEC: usize = 221;
pub const SYSCALL_WAIT4: usize = 260;
#[no_mangle]
// #[cfg(feature = "user")]
pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => syscall_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => syscall_exit(args[0] as i32),
        SYSCALL_EXEC => syscall_exec(args[0] as *const u8, args[1] as *const usize),
        SYSCALL_CLONE => syscall_clone(args[0], args[1], args[2], args[3], args[4]),
        SYSCALL_GETPID => syscall_getpid(),
        SYSCALL_GETPPID => syscall_getppid(),
        SYSCALL_WAIT4 => syscall_wait4(
            args[0] as isize,
            args[1] as *mut i32,
            WaitFlags::from_bits(args[2] as u32).unwrap(),
        ),
        _ => {
            panic!("Invalid Syscall Id: {}!", syscall_id);
        }
    }
}
