//! 系统调用号

/// 文件系统
/// 获取当前工作目录
pub const SYSCALL_GETCWD: usize = 17;
/// 复制文件描述符
pub const SYSCALL_DUP: usize = 23;
/// 复制文件描述符
pub const SYSCALL_DUP3: usize = 24;
/// 创建目录
pub const SYSCALL_MKDIRAT: usize = 34;
/// 删除文件
pub const SYSCALL_UNLINKAT: usize = 35;
/// 创建硬链接
pub const SYSCALL_LINKAT: usize = 37;
/// 创建符号链接
pub const SYSCALL_UNMOUNT: usize = 39;
/// 挂载文件系统
pub const SYSCALL_MOUNT: usize = 40;
/// 改变当前工作目录
pub const SYSCALL_CHDIR: usize = 49;
/// 打开文件
pub const SYSCALL_OPENAT: usize = 56;
/// 关闭文件
pub const SYSCALL_CLOSE: usize = 57;
/// 创建管道
pub const SYSCALL_PIPE2: usize = 59;
/// 读取目录
pub const SYSCALL_GETDENTS64: usize = 61;
/// 读取文件
pub const SYSCALL_READ: usize = 63;
/// 写入文件
pub const SYSCALL_WRITE: usize = 64;
/// 获取文件信息
pub const SYSCALL_FSTAT: usize = 80;

/// 进程管理
/// 退出进程
pub const SYSCALL_EXIT: usize = 93;
/// 获取进程id
pub const SYSCALL_GETPID: usize = 172;
/// 获取父进程id
pub const SYSCALL_GETPPID: usize = 173;
/// 创建进程
pub const SYSCALL_CLONE: usize = 220;
/// 执行程序
pub const SYSCALL_EXECVE: usize = 221;
/// 等待子进程退出
pub const SYSCALL_WAIT4: usize = 260;

/// 内存管理
/// 修改数据段大小
pub const SYSCALL_BRK: usize = 214;
/// 释放内存
pub const SYSCALL_MUNMAP: usize = 215;
/// 修改内存保护属性
pub const SYSCALL_MMAP: usize = 222;

/// 其他
/// 纳秒级延迟
pub const SYSCALL_NANO_SLEEP: usize = 101;
/// yield
pub const SYSCALL_SCHED_YIELD: usize = 124;
//?159
/// 获取时间
pub const SYSCALL_TIMES: usize = 153;
/// 获取系统信息
pub const SYSCALL_UNAME: usize = 160;
/// 获取时间
pub const SYSCALL_GETTIMEOFDAY: usize = 169;

/// 从syscall_id获取syscall_name
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
