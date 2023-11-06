//! 记录该模块使用到的系统调用 id
//!
//!
numeric_enum_macro::numeric_enum! {
#[repr(usize)]
#[allow(non_camel_case_types)]
#[allow(missing_docs)]
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum TaskSyscallId {
    EXIT = 93,
    EXIT_GROUP = 94,
    SET_TID_ADDRESS = 96,
    FUTEX = 98,
    SET_ROBUST_LIST = 99,
    GET_ROBUST_LIST = 100,
    NANO_SLEEP = 101,
    GETTIMER = 102,
    SETITIMER = 103,
    CLOCK_GETRES = 114,
    CLOCK_NANOSLEEP = 115,
    SYSLOG = 116,
    SCHED_SETSCHEDULER = 119,
    SCHED_GETSCHEDULER = 120,
    SCHED_SETAFFINITY = 122,
    SCHED_GETAFFINITY = 123,
    SETSID = 157,
    GETRUSAGE = 165,
    UMASK = 166,
    PRCTL = 167,
    GETPID = 172,
    GETPPID = 173,
    GETUID = 174,
    GETEUID = 175,
    GETGID = 176,
    GETEGID = 177,
    GETTID = 178,
    SYSINFO = 179,
    SOCKETPAIR = 199,
    CLONE = 220,
    EXECVE = 221,
    MADVICE = 233,
    WAIT4 = 260,
    GETRANDOM = 278,
    SCHED_YIELD = 124,
    CLOCK_GET_TIME = 113,
    SIGTIMEDWAIT = 137,
    TIMES = 153,
    UNAME = 160,
    GETTIMEOFDAY = 169,
    PRLIMIT64 = 261,
    // 信号模块
    KILL = 129,
    TKILL = 130,
    SIGSUSPEND = 133,
    SIGACTION = 134,
    SIGPROCMASK = 135,
    SIGRETURN = 139,
}
}
