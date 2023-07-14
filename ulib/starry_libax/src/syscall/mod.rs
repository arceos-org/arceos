use axfs::monolithic_fs::file_io::Kstat;
use axprocess::process::exit;
use axsignal::action::SigAction;
use axtask::current;
use flags::{MMAPFlags, TimeSecs, TimeVal, UtsName, WaitFlags, MMAPPROT, TMS};
use fs::*;
use log::{debug, error, info};
use mem::{syscall_brk, syscall_mmap, syscall_msync, syscall_munmap};
use task::*;
use utils::*;
extern crate axlog;
extern crate log;

extern crate alloc;

pub mod flags;
pub mod fs;
pub mod futex;
pub mod mem;
#[cfg(feature = "signal")]
pub mod signal;
pub mod socket;
pub mod syscall_id;
pub mod utils;
#[allow(unused)]
use syscall_id::*;

use crate::syscall::{
    flags::{IoVec, RLimit, SigMaskFlag},
    mem::syscall_mprotect,
    signal::{
        syscall_kill, syscall_sigaction, syscall_sigprocmask, syscall_sigreturn, syscall_tkill,
    },
    socket::{
        syscall_bind, syscall_get_sock_name, syscall_listen, syscall_recvfrom, syscall_sendto,
        syscall_set_sock_opt, syscall_socket,
    },
};

pub mod task;

#[no_mangle]
pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    let curr_id = current().id().as_u64();
    info!(
        "task id: {}, syscall: id: {}, name: {}",
        curr_id,
        syscall_id,
        get_syscall_name(syscall_id)
    );
    let ans = match syscall_id {
        SYSCALL_OPENAT => syscall_openat(
            args[0],
            args[1] as *const u8,
            args[2] as usize,
            args[3] as u8,
        ), // args[0] is fd, args[1] is filename, args[2] is flags, args[3] is mode
        SYSCALL_CLOSE => syscall_close(args[0]), // args[0] is fd
        // SYSCALL_GETDENTS64 => syscall_getdents64(args[0], args[1] as *mut u8, args[2]),
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
            args[4] as usize,
            args[5],
        ),
        SYSCALL_MSYNC => syscall_msync(args[0], args[1]),
        SYSCALL_GETCWD => syscall_getcwd(args[0] as *mut u8, args[1]),
        SYSCALL_PIPE2 => syscall_pipe2(args[0] as *mut u32),
        SYSCALL_DUP => syscall_dup(args[0]),
        SYSCALL_DUP3 => syscall_dup3(args[0], args[1]),
        SYSCALL_MKDIRAT => syscall_mkdirat(args[0], args[1] as *const u8, args[2] as u32),
        SYSCALL_CHDIR => syscall_chdir(args[0] as *const u8),
        SYSCALL_GETDENTS64 => {
            debug!("syscall_getdents64");
            syscall_getdents64(args[0], args[1] as *mut u8, args[2] as usize)
        }
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

        SYSCALL_SIGACTION => syscall_sigaction(
            args[0],
            args[1] as *const SigAction,
            args[2] as *mut SigAction,
        ),

        SYSCALL_KILL => syscall_kill(args[0] as isize, args[1] as isize),
        SYSCALL_TKILL => syscall_tkill(args[0] as isize, args[1] as isize),
        SYSCALL_SIGPROCMASK => syscall_sigprocmask(
            SigMaskFlag::from(args[0]),
            args[1] as *const usize,
            args[2] as *mut usize,
            args[3],
        ),
        SYSCALL_SIGRETURN => syscall_sigreturn(),
        SYSCALL_EXIT_GROUP => syscall_exit(args[0] as i32),
        SYSCALL_SET_TID_ADDRESS => syscall_set_tid_address(args[0] as usize),
        SYSCALL_PRLIMIT64 => syscall_prlimit64(
            args[0] as usize,
            args[1] as i32,
            args[2] as *const RLimit,
            args[3] as *mut RLimit,
        ),
        SYSCALL_CLOCK_GET_TIME => {
            syscall_clock_get_time(args[0] as usize, args[1] as *mut TimeSecs)
        }
        SYSCALL_GETUID => syscall_getuid(),
        SYSCALL_GETEUID => syscall_geteuid(),
        SYSCALL_GETGID => syscall_getgid(),
        SYSCALL_GETEGID => syscall_getegid(),
        SYSCALL_GETTID => syscall_gettid(),
        SYSCALL_FUTEX => syscall_futex(
            args[0] as usize,
            args[1] as i32,
            args[2] as u32,
            args[3] as usize,
            args[4] as usize,
            args[5] as u32,
        ),
        SYSCALL_SET_ROBUST_LIST => syscall_set_robust_list(args[0] as usize, args[1] as usize),
        SYSCALL_GET_ROBUST_LIST => {
            syscall_get_robust_list(args[0] as i32, args[1] as *mut usize, args[2] as *mut usize)
        }
        SYSCALL_READV => syscall_readv(args[0] as usize, args[1] as *mut IoVec, args[2] as usize),
        SYSCALL_WRITEV => {
            syscall_writev(args[0] as usize, args[1] as *const IoVec, args[2] as usize)
        }
        SYSCALL_MPROTECT => syscall_mprotect(
            args[0] as usize,
            args[1] as usize,
            MMAPPROT::from_bits_truncate(args[2] as u32),
        ),
        SYSCALL_FCNTL64 => syscall_fcntl64(args[0] as usize, args[1] as usize, args[2] as usize),

        SYSCALL_SOCKET => syscall_socket(args[0], args[1], args[2]),
        SYSCALL_BIND => syscall_bind(args[0], args[1] as *const u8, args[2]),
        SYSCALL_LISTEN => syscall_listen(args[0], args[1]),
        SYSCALL_GETSOCKNAME => {
            syscall_get_sock_name(args[0], args[1] as *mut u8, args[2] as *mut usize)
        }
        SYSCALL_SENDTO => syscall_sendto(
            args[0],
            args[1] as *const u8,
            args[2],
            args[3],
            args[4] as *const u8,
            args[5],
        ),
        SYSCALL_RECVFROM => syscall_recvfrom(
            args[0],
            args[1] as *mut u8,
            args[2],
            args[3],
            args[4] as *mut u8,
            args[5] as *mut usize,
        ),
        SYSCALL_SETSOCKOPT => {
            syscall_set_sock_opt(args[0], args[1], args[2], args[3] as *const u8, args[4])
        }

        _ => {
            error!("Invalid Syscall Id: {}!", syscall_id);
            // return -1;
            exit(-1)
        }
    };
    // info!(
    //     "currr id: {}, Syscall {} return: {}",
    //     curr_id, syscall_id, ans
    // );
    ans
}
