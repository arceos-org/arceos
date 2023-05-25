#![cfg_attr(not(test), no_std)]

use axfs_os::types::Kstat;
use flags::{MMAPFlags, TimeSecs, TimeVal, UtsName, WaitFlags, MMAPPROT, TMS};
use fs::*;
use mem::{syscall_brk, syscall_mmap, syscall_munmap};
use task::*;

extern crate axlog;
extern crate log;

extern crate alloc;
mod flags;
mod fs;
mod mem;
mod syscall_id;
#[allow(unused)]
use syscall_id::*;

mod task;
#[no_mangle]
// #[cfg(feature = "user")]
pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    axlog::info!("syscall: {}", get_syscall_name(syscall_id));
    axlog::info!("args: {:?}", args);
    match syscall_id {
        SYSCALL_OPENAT => syscall_openat(args[0], args[1] as *const u8, args[2] as usize, args[3] as u8), // args[0] is fd, args[1] is filename, args[2] is flags, args[3] is mode
        SYSCALL_CLOSE => syscall_close(args[0]), // args[0] is fd
        SYSCALL_READ => syscall_read(args[0], args[1] as *mut u8, args[2]),
        SYSCALL_WRITE => syscall_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => syscall_exit(args[0] as i32),
        SYSCALL_EXECVE => syscall_exec(args[0] as *const u8, args[1] as *const usize),
        SYSCALL_CLONE => syscall_clone(args[0], args[1], args[2], args[3], args[4]),
        SYSCALL_NANO_SLEEP => syscall_sleep(args[0] as *const TimeSecs, args[1] as *mut TimeSecs),
        SYSCALL_SCHED_YIELD => syscall_yield(),
        SYSCALL_TIMES => syscall_time(args[0] as *mut TMS),
        SYSCALL_UNAME => syscall_uname(args[0] as *mut UtsName),
        SYSCALL_GETTIMEOFDAY => syscall_get_time_of_day(args[0] as *mut TimeVal),
        SYSCALL_GETPID => syscall_getpid(),
        SYSCALL_GETPPID => syscall_getppid(),
        SYSCALL_WAIT4 => syscall_wait4(
            args[0] as isize,
            args[1] as *mut i32,
            WaitFlags::from_bits(args[2] as u32).unwrap(),
        ),
        SYSCALL_BRK => syscall_brk(args[0] as usize),
        SYSCALL_MUNMAP => syscall_munmap(args[0], args[1]),
        SYSCALL_MMAP => syscall_mmap(
            args[0],
            args[1],
            MMAPPROT::from_bits_truncate(args[2] as u32),
            MMAPFlags::from_bits_truncate(args[3] as u32),
            args[4] as i32,
            args[5],
        ),
        SYSCALL_GETCWD => syscall_getcwd(args[0] as *mut u8, args[1]),
        SYSCALL_PIPE2 => syscall_pipe2(args[0] as *mut u32),
        SYSCALL_DUP => syscall_dup(args[0]),
        SYSCALL_DUP3 => syscall_dup3(args[0], args[1]),
        SYSCALL_MKDIRAT => syscall_mkdirat(args[0], args[1] as *const u8, args[2] as u32),
        SYSCALL_CHDIR => syscall_chdir(args[0] as *const u8),
        SYSCALL_GETDENTS64 => syscall_getdents64(args[0], args[1] as *mut u8, args[2] as usize),
        SYSCALL_UNLINKAT => syscall_unlinkat(args[0], args[1] as *const u8, args[2] as usize),
        SYSCALL_MOUNT => syscall_mount(
            args[0] as *const u8,
            args[1] as *const u8,
            args[2] as *const u8,
            args[3] as usize,
            args[4] as *const u8,
        ),
        SYSCALL_UNMOUNT => syscall_umount(args[0] as *const u8, args[1] as usize),
        SYSCALL_FSTAT => syscall_fstat(args[0], args[1] as *mut Kstat),

        _ => {
            panic!("Invalid Syscall Id: {}!", syscall_id);
        }
    }
}
