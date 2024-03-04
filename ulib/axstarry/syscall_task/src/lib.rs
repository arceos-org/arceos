//! 提供和 task 模块相关的 syscall
#![cfg_attr(all(not(test), not(doc)), no_std)]

mod task_syscall_id;

use syscall_utils::SyscallResult;
pub use task_syscall_id::TaskSyscallId::{self, *};

mod imp;

pub use imp::*;

/// 进行 syscall 的分发
pub fn task_syscall(syscall_id: task_syscall_id::TaskSyscallId, args: [usize; 6]) -> SyscallResult {
    match syscall_id {
        EXIT => syscall_exit(args),
        EXECVE => syscall_exec(args),
        CLONE => syscall_clone(args),
        CLONE3 => syscall_clone3(args),
        NANO_SLEEP => syscall_sleep(args),
        SCHED_YIELD => syscall_yield(),
        TIMES => syscall_time(args),
        UNAME => syscall_uname(args),
        GETTIMEOFDAY => syscall_get_time_of_day(args),
        GETPID => syscall_getpid(),
        GETPPID => syscall_getppid(),
        WAIT4 => syscall_wait4(args),
        GETRANDOM => syscall_getrandom(args),
        #[cfg(feature = "signal")]
        SIGSUSPEND => syscall_sigsuspend(args),
        #[cfg(feature = "signal")]
        SIGACTION => syscall_sigaction(args),
        #[cfg(feature = "signal")]
        KILL => syscall_kill(args),
        #[cfg(feature = "signal")]
        TKILL => syscall_tkill(args),
        #[cfg(feature = "signal")]
        TGKILL => syscall_tkill(args),
        #[cfg(feature = "signal")]
        SIGPROCMASK => syscall_sigprocmask(args),
        #[cfg(feature = "signal")]
        SIGRETURN => syscall_sigreturn(),
        EXIT_GROUP => syscall_exit(args),
        SET_TID_ADDRESS => syscall_set_tid_address(args),
        PRLIMIT64 => syscall_prlimit64(args),
        CLOCK_GET_TIME => syscall_clock_get_time(args),
        GETUID => syscall_getuid(),
        GETEUID => syscall_geteuid(),
        GETGID => syscall_getgid(),
        GETEGID => syscall_getegid(),
        GETTID => syscall_gettid(),
        #[cfg(feature = "futex")]
        FUTEX => syscall_futex(args),
        #[cfg(feature = "futex")]
        SET_ROBUST_LIST => syscall_set_robust_list(args),
        #[cfg(feature = "futex")]
        GET_ROBUST_LIST => syscall_get_robust_list(args),
        SYSINFO => syscall_sysinfo(args),
        SETITIMER => syscall_settimer(args),
        GETTIMER => syscall_gettimer(args),
        SETSID => syscall_setsid(),
        GETRUSAGE => syscall_getrusage(args),
        UMASK => syscall_umask(args),
        // 不做处理即可
        SIGTIMEDWAIT => Ok(0),
        SYSLOG => Ok(0),
        MADVICE => Ok(0),
        #[cfg(feature = "schedule")]
        SCHED_SETAFFINITY => Ok(0),
        #[cfg(feature = "schedule")]
        SCHED_GETAFFINITY => syscall_sched_getaffinity(args),
        #[cfg(feature = "schedule")]
        SCHED_SETSCHEDULER => syscall_sched_setscheduler(args),
        #[cfg(feature = "schedule")]
        SCHED_GETSCHEDULER => syscall_sched_getscheduler(args),
        #[cfg(feature = "schedule")]
        GET_MEMPOLICY => Ok(0),
        CLOCK_GETRES => syscall_clock_getres(args),
        CLOCK_NANOSLEEP => syscall_clock_nanosleep(args),
        // syscall below just for x86_64
        #[cfg(target_arch = "x86_64")]
        PRCTL => syscall_prctl(args),
        #[cfg(target_arch = "x86_64")]
        VFORK => syscall_vfork(),
        #[cfg(target_arch = "x86_64")]
        ARCH_PRCTL => syscall_arch_prctl(args),
        #[cfg(target_arch = "x86_64")]
        FORK => syscall_fork(),
        #[cfg(target_arch = "x86_64")]
        GETPGID => syscall_getpgid(),
        #[cfg(target_arch = "x86_64")]
        SETPGID => syscall_setpgid(),
        #[cfg(target_arch = "x86_64")]
        ALARM => Ok(0),
        #[cfg(target_arch = "x86_64")]
        RSEQ => Ok(0),
        #[allow(unused)]
        _ => {
            panic!("Invalid Syscall Id: {:?}!", syscall_id);
            // return -1;
            // exit(-1)
        }
    }
}
