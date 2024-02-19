//! 提供和 task 模块相关的 syscall
#![cfg_attr(all(not(test), not(doc)), no_std)]

mod task_syscall_id;
#[cfg(feature = "signal")]
use axsignal::action::SigAction;
use syscall_utils::{
    ITimerVal, RLimit, SysInfo, SyscallError, SyscallResult, TimeSecs, TimeVal, UtsName, WaitFlags,
    TMS,
};
pub use task_syscall_id::TaskSyscallId::{self, *};

#[cfg(feature = "schedule")]
use syscall_utils::SchedParam;

mod imp;

pub use imp::*;

/// 进行 syscall 的分发
pub fn task_syscall(syscall_id: task_syscall_id::TaskSyscallId, args: [usize; 6]) -> SyscallResult {
    match syscall_id {
        EXIT => syscall_exit(args[0] as i32),
        EXECVE => syscall_exec(
            args[0] as *const u8,
            args[1] as *const usize,
            args[2] as *const usize,
        ),
        CLONE => syscall_clone(args[0], args[1], args[2], args[3], args[4]),
        VFORK => syscall_vfork(),
        NANO_SLEEP => syscall_sleep(args[0] as *const TimeSecs, args[1] as *mut TimeSecs),
        SCHED_YIELD => syscall_yield(),
        TIMES => syscall_time(args[0] as *mut TMS),
        UNAME => syscall_uname(args[0] as *mut UtsName),
        GETTIMEOFDAY => syscall_get_time_of_day(args[0] as *mut TimeVal),
        GETPID => syscall_getpid(),
        GETPPID => syscall_getppid(),
        WAIT4 => syscall_wait4(
            args[0] as isize,
            args[1] as *mut i32,
            WaitFlags::from_bits(args[2] as u32).unwrap(),
        ),
        GETRANDOM => syscall_getrandom(args[0] as *mut u8, args[1], args[2]),
        #[cfg(feature = "signal")]
        SIGSUSPEND => syscall_sigsuspend(args[0] as *const usize),
        #[cfg(feature = "signal")]
        SIGACTION => syscall_sigaction(
            args[0],
            args[1] as *const SigAction,
            args[2] as *mut SigAction,
        ),
        #[cfg(feature = "signal")]
        KILL => syscall_kill(args[0] as isize, args[1] as isize),
        #[cfg(feature = "signal")]
        TKILL => syscall_tkill(args[0] as isize, args[1] as isize),
        #[cfg(feature = "signal")]
        SIGPROCMASK => syscall_sigprocmask(
            syscall_utils::SigMaskFlag::from(args[0]),
            args[1] as *const usize,
            args[2] as *mut usize,
            args[3],
        ),
        #[cfg(feature = "signal")]
        SIGRETURN => syscall_sigreturn(),
        EXIT_GROUP => syscall_exit(args[0] as i32),
        SET_TID_ADDRESS => syscall_set_tid_address(args[0] as usize),
        PRLIMIT64 => syscall_prlimit64(
            args[0] as usize,
            args[1] as i32,
            args[2] as *const RLimit,
            args[3] as *mut RLimit,
        ),
        CLOCK_GET_TIME => syscall_clock_get_time(args[0] as usize, args[1] as *mut TimeSecs),
        GETUID => syscall_getuid(),
        GETEUID => syscall_geteuid(),
        GETGID => syscall_getgid(),
        GETEGID => syscall_getegid(),
        GETTID => syscall_gettid(),
        #[cfg(feature = "futex")]
        FUTEX => syscall_futex(
            args[0] as usize,
            args[1] as i32,
            args[2] as u32,
            args[3] as usize,
            args[4] as usize,
            args[5] as u32,
        ),
        #[cfg(feature = "futex")]
        SET_ROBUST_LIST => syscall_set_robust_list(args[0] as usize, args[1] as usize),
        #[cfg(feature = "futex")]
        GET_ROBUST_LIST => {
            syscall_get_robust_list(args[0] as i32, args[1] as *mut usize, args[2] as *mut usize)
        }
        SYSINFO => syscall_sysinfo(args[0] as *mut SysInfo),
        SETITIMER => syscall_settimer(
            args[0] as usize,
            args[1] as *const ITimerVal,
            args[2] as *mut ITimerVal,
        ),
        GETTIMER => syscall_gettimer(args[0] as usize, args[1] as *mut ITimerVal),
        SETSID => syscall_setsid(),
        GETRUSAGE => syscall_getrusage(args[0] as i32, args[1] as *mut TimeVal),
        UMASK => syscall_umask(args[0] as i32),
        // 不做处理即可
        SIGTIMEDWAIT => Ok(0),
        SYSLOG => Ok(0),
        PRCTL => syscall_prctl(args[0], args[1] as *mut u8),
        MADVICE => Ok(0),
        #[cfg(feature = "schedule")]
        SCHED_SETAFFINITY => Ok(0),
        #[cfg(feature = "schedule")]
        SCHED_GETAFFINITY => {
            syscall_sched_getaffinity(args[0] as usize, args[1] as usize, args[2] as *mut usize)
        }
        #[cfg(feature = "schedule")]
        SCHED_SETSCHEDULER => syscall_sched_setscheduler(
            args[0] as usize,
            args[1] as usize,
            args[2] as *const SchedParam,
        ),
        #[cfg(feature = "schedule")]
        SCHED_GETSCHEDULER => syscall_sched_getscheduler(args[0] as usize),
        CLOCK_GETRES => syscall_clock_getres(args[0] as usize, args[1] as *mut TimeSecs),
        CLOCK_NANOSLEEP => syscall_clock_nanosleep(
            args[0] as usize,
            args[1] as usize,
            args[2] as *const TimeSecs,
            args[3] as *mut TimeSecs,
        ),
        SOCKETPAIR => Err(SyscallError::EAFNOSUPPORT),
        // syscall below just for x86_64 
        #[cfg(target_arch = "x86_64")]
        ARCH_PRCTL => syscall_arch_prctl(args[0], args[1]),
        #[cfg(target_arch = "x86_64")]
        FORK => syscall_fork(),
        #[cfg(target_arch = "x86_64")]
        GETPGID => syscall_getpgid(),
        #[cfg(target_arch = "x86_64")]
        SETPGID => syscall_setpgid(),
        #[cfg(target_arch = "x86_64")]
        ALARM => Ok(0),
        RSEQ => Ok(0),
        #[allow(unused)]
        _ => {
            panic!("Invalid Syscall Id: {:?}!", syscall_id);
            // return -1;
            // exit(-1)
        }
    }
}
