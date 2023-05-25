// 文件系统
pub const SYSCALL_GETCWD: usize = 17;
pub const SYSCALL_DUP: usize = 23;
pub const SYSCALL_DUP3: usize = 24;
//?
pub const SYSCALL_MKDIRAT: usize = 34;
pub const SYSCALL_UNLINKAT: usize = 35;
pub const SYSCALL_LINKAT: usize = 37;
pub const SYSCALL_UNMOUNT: usize = 39;
pub const SYSCALL_MOUNT: usize = 40;
pub const SYSCALL_CHDIR: usize = 49;
pub const SYSCALL_OPENAT: usize = 56;
pub const SYSCALL_CLOSE: usize = 57;
pub const SYSCALL_PIPE2: usize = 59;
pub const SYSCALL_GETDENTS64: usize = 61;
pub const SYSCALL_READ: usize = 63;
pub const SYSCALL_WRITE: usize = 64;
pub const SYSCALL_FSTAT: usize = 80;

// 进程管理
pub const SYSCALL_EXIT: usize = 93;
pub const SYSCALL_GETPID: usize = 172;
pub const SYSCALL_GETPPID: usize = 173;
pub const SYSCALL_CLONE: usize = 220;
pub const SYSCALL_EXECVE: usize = 221;
pub const SYSCALL_WAIT4: usize = 260;

// 内存管理
pub const SYSCALL_BRK: usize = 214;
pub const SYSCALL_MUNMAP: usize = 215;
pub const SYSCALL_MMAP: usize = 222;

// 其他
pub const SYSCALL_NANO_SLEEP: usize = 101;
pub const SYSCALL_SCHED_YIELD: usize = 124;
//?159
pub const SYSCALL_TIMES: usize = 153;
pub const SYSCALL_UNAME: usize = 160;
pub const SYSCALL_GETTIMEOFDAY: usize = 169;

// 从syscall_id获取syscall_name
pub fn get_syscall_name(syscall_id: usize) -> &'static str {
    match syscall_id {
        SYSCALL_GETCWD => "getcwd",
        SYSCALL_DUP => "dup",
        SYSCALL_DUP3 => "dup3",
        SYSCALL_MKDIRAT => "mkdirat",
        SYSCALL_UNLINKAT => "unlinkat",
        SYSCALL_LINKAT => "linkat",
        SYSCALL_UNMOUNT => "unmount",
        SYSCALL_MOUNT => "mount",
        SYSCALL_CHDIR => "chdir",
        SYSCALL_OPENAT => "openat",
        SYSCALL_CLOSE => "close",
        SYSCALL_PIPE2 => "pipe2",
        SYSCALL_GETDENTS64 => "getdents64",
        SYSCALL_READ => "read",
        SYSCALL_WRITE => "write",
        SYSCALL_FSTAT => "fstat",
        SYSCALL_EXIT => "exit",
        SYSCALL_GETPID => "getpid",
        SYSCALL_GETPPID => "getppid",
        SYSCALL_CLONE => "clone",
        SYSCALL_EXECVE => "execve",
        SYSCALL_WAIT4 => "wait4",
        SYSCALL_BRK => "brk",
        SYSCALL_MUNMAP => "munmap",
        SYSCALL_MMAP => "mmap",
        SYSCALL_NANO_SLEEP => "nanosleep",
        SYSCALL_SCHED_YIELD => "sched_yield",
        SYSCALL_TIMES => "times",
        SYSCALL_UNAME => "uname",
        SYSCALL_GETTIMEOFDAY => "gettimeofday",
        _ => "unknown",
    }
}