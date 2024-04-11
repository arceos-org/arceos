//! 记录该模块使用到的系统调用 id
//!
//!
//!
#[cfg(any(
    target_arch = "riscv32",
    target_arch = "riscv64",
    target_arch = "aarch64"
))]
numeric_enum_macro::numeric_enum! {
#[repr(usize)]
#[allow(non_camel_case_types)]
#[allow(missing_docs)]
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum FsSyscallId {
    // fs
    GETCWD = 17,
    EVENT_FD = 19,
    EPOLL_CREATE = 20,
    EPOLL_CTL = 21,
    EPOLL_WAIT = 22,
    DUP = 23,
    DUP3 = 24,
    FCNTL64 = 25,
    IOCTL = 29,
    MKDIRAT = 34,
    UNLINKAT = 35,
    LINKAT = 37,
    RENAMEAT = 38,
    UNMOUNT = 39,
    MOUNT = 40,
    STATFS = 43,
    FTRUNCATE64 = 46,
    FACCESSAT = 48,
    CHDIR = 49,
    FCHMODAT = 53,
    OPENAT = 56,
    CLOSE = 57,
    PIPE2 = 59,
    GETDENTS64 = 61,
    LSEEK = 62,
    READ = 63,
    WRITE = 64,
    READV = 65,
    WRITEV = 66,
    PPOLL = 73,
    FSTATAT = 79,
    PREAD64 = 67,
    PWRITE64 = 68,
    SENDFILE64 = 71,
    PSELECT6 = 72,
    PREADLINKAT = 78,
    FSTAT = 80,
    SYNC = 81,
    FSYNC = 82,
    UTIMENSAT = 88,
    RENAMEAT2 = 276,
    COPYFILERANGE = 285,
}
}

#[cfg(target_arch = "x86_64")]
numeric_enum_macro::numeric_enum! {
    #[repr(usize)]
    #[allow(non_camel_case_types)]
    #[allow(missing_docs)]
    #[derive(Eq, PartialEq, Debug, Copy, Clone)]
    pub enum FsSyscallId {
        // fs
        OPEN = 2,
        STAT = 4,
        EVENT_FD = 284,
        GETCWD = 79,
        UNLINK = 87,
        EPOLL_CREATE = 213,
        EPOLL_CTL = 233,
        EPOLL_WAIT = 232,
        DUP = 32,
        DUP2 = 33,
        DUP3 = 292,
        FCNTL64 = 72,
        IOCTL = 16,
        MKDIRAT = 258,
        RENAME = 82,
        MKDIR = 83,
        RMDIR = 84,
        UNLINKAT = 263,
        LINKAT = 265,
        UNMOUNT = 166,
        MOUNT = 165,
        STATFS = 137,
        FTRUNCATE64 = 77,
        FACCESSAT = 269,
        ACCESS = 21,
        CHDIR = 80,
        FCHMODAT = 268,
        OPENAT = 257,
        CLOSE = 3,
        PIPE = 22,
        PIPE2 = 293,
        GETDENTS64 = 217,
        LSEEK = 8,
        READ = 0,
        WRITE = 1,
        READV = 19,
        WRITEV = 20,
        PPOLL = 271,
        POLL = 7,
        CREAT = 85,
        FSTATAT = 262,
        PREAD64 = 17,
        PWRITE64 = 18,
        SENDFILE64 = 40,
        SELECT = 23,
        PSELECT6 = 270,
        READLINK = 89,
        PREADLINKAT = 267,
        FSTAT = 5,
        LSTAT = 6,
        SYNC = 162,
        FSYNC = 74,
        UTIMENSAT = 280,
        RENAMEAT = 264,
        RENAMEAT2 = 316,
        COPYFILERANGE = 326,
    }
}
