// 文件系统

numeric_enum_macro::numeric_enum! {
#[repr(usize)]
#[allow(non_camel_case_types)]
#[allow(missing_docs)]
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum SyscallId {
    UNKNOWN = 0,
    // 文件系统
    GETCWD = 17,
    DUP = 23,
    DUP3 = 24,
    FCNTL64 = 25,
    MKDIRAT = 34,
    UNLINKAT = 35,
    LINKAT = 37,
    UNMOUNT = 39,
    MOUNT = 40,
    CHDIR = 49,
    OPENAT = 56,
    CLOSE = 57,
    PIPE2 = 59,
    GETDENTS64 = 61,
    READ = 63,
    WRITE = 64,
    READV = 65,
    WRITEV = 66,
    FSTAT = 80,
    EXIT = 93,
    EXIT_GROUP = 94,
    SET_TID_ADDRESS = 96,
    FUTEX = 98,
    SET_ROBUST_LIST = 99,
    GET_ROBUST_LIST = 100,
    GETPID = 172,
    GETPPID = 173,
    GETUID = 174,
    GETEUID = 175,
    GETGID = 176,
    GETEGID = 177,
    GETTID = 178,
    CLONE = 220,
    EXECVE = 221,
    WAIT4 = 260,
    // 内存管理
    BRK = 214,
    MUNMAP = 215,
    MMAP = 222,
    MSYNC = 227,
    MPROTECT = 226,
    // 其他
    NANO_SLEEP = 101,
    SCHED_YIELD = 124,
    CLOCK_GET_TIME = 113,
    TIMES = 153,
    UNAME = 160,
    GETTIMEOFDAY = 169,
    PRLIMIT64 = 261,
    // 信号模块
    KILL = 129,
    TKILL = 130,
    SIGACTION = 134,
    SIGPROCMASK = 135,
    SIGRETURN = 139,
}
}

/// 系统调用错误编号
#[repr(C)]
#[derive(Debug)]
pub enum ErrorNo {
    /// 非法操作
    EPERM = -1,
    /// 找不到文件或目录
    ENOENT = -2,
    /// 找不到对应进程
    ESRCH = -3,
    /// 错误的文件描述符
    EBADF = -9,
    /// 资源暂时不可用。也可因为 futex_wait 时对应用户地址处的值与给定值不符
    EAGAIN = -11,
    /// 内存耗尽，或者没有对应的内存映射
    ENOMEM = -12,
    /// 无效地址
    EFAULT = -14,
    /// 设备或者资源被占用
    EBUSY = -16,
    /// 文件已存在
    EEXIST = -17,
    /// 不是一个目录(但要求需要是一个目录)
    ENOTDIR = -20,
    /// 是一个目录(但要求不能是)
    EISDIR = -21,
    /// 非法参数
    EINVAL = -22,
    /// fd（文件描述符）已满
    EMFILE = -24,
    /// 对文件进行了无效的 seek
    ESPIPE = -29,
    /// 超过范围。例如用户提供的buffer不够长
    ERANGE = -34,
    /// 不支持的协议
    EPFNOSUPPORT = -96,
    /// 不支持的地址
    EAFNOSUPPORT = -97,
    /// 拒绝连接
    ECONNREFUSED = -111,
}
