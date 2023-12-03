#![cfg_attr(all(not(test), not(doc)), no_std)]
mod ctype;
pub mod imp;
use axfs::api::Kstat;
use ctype::epoll::EpollEvent;
pub use ctype::FileDesc;
use syscall_utils::{FsStat, IoVec, SyscallResult, TimeSecs};
mod fs_syscall_id;
pub use fs_syscall_id::FsSyscallId;
use fs_syscall_id::FsSyscallId::*;
use imp::*;
pub fn fs_syscall(syscall_id: fs_syscall_id::FsSyscallId, args: [usize; 6]) -> SyscallResult {
    match syscall_id {
        OPENAT => syscall_openat(
            args[0],
            args[1] as *const u8,
            args[2] as usize,
            args[3] as u8,
        ),
        CLOSE => syscall_close(args[0]),
        READ => syscall_read(args[0], args[1] as *mut u8, args[2]),
        WRITE => syscall_write(args[0], args[1] as *const u8, args[2]),
        GETCWD => syscall_getcwd(args[0] as *mut u8, args[1]),
        PIPE2 => syscall_pipe2(args[0] as *mut u32, args[1]),
        DUP => syscall_dup(args[0]),
        DUP3 => syscall_dup3(args[0], args[1]),
        MKDIRAT => syscall_mkdirat(args[0], args[1] as *const u8, args[2] as u32),
        CHDIR => syscall_chdir(args[0] as *const u8),
        GETDENTS64 => syscall_getdents64(args[0], args[1] as *mut u8, args[2] as usize),
        MOUNT => syscall_mount(
            args[0] as *const u8,
            args[1] as *const u8,
            args[2] as *const u8,
            args[3] as usize,
            args[4] as *const u8,
        ),
        UNMOUNT => syscall_umount(args[0] as *const u8, args[1] as usize),
        FSTAT => syscall_fstat(args[0], args[1] as *mut Kstat),
        RENAMEAT2 => syscall_renameat2(
            args[0],
            args[1] as *const u8,
            args[2],
            args[3] as *const u8,
            args[4],
        ),
        READV => syscall_readv(args[0] as usize, args[1] as *mut IoVec, args[2] as usize),
        WRITEV => syscall_writev(args[0] as usize, args[1] as *const IoVec, args[2] as usize),
        FCNTL64 => syscall_fcntl64(args[0] as usize, args[1] as usize, args[2] as usize),
        FSTATAT => syscall_fstatat(
            args[0] as usize,
            args[1] as *const u8,
            args[2] as *mut Kstat,
        ),
        STATFS => syscall_statfs(args[0] as *const u8, args[1] as *mut FsStat),
        FCHMODAT => syscall_fchmodat(args[0] as usize, args[1] as *const u8, args[2] as usize),
        FACCESSAT => syscall_faccessat(args[0] as usize, args[1] as *const u8, args[2] as usize),
        LSEEK => syscall_lseek(args[0] as usize, args[1] as isize, args[2] as usize),
        PREAD64 => syscall_pread64(
            args[0] as usize,
            args[1] as *mut u8,
            args[2] as usize,
            args[3] as usize,
        ),
        PREADLINKAT => syscall_readlinkat(
            args[0] as usize,
            args[1] as *const u8,
            args[2] as *mut u8,
            args[3] as usize,
        ),
        PWRITE64 => syscall_pwrite64(args[0], args[1] as *const u8, args[2], args[3]),
        SENDFILE64 => syscall_sendfile64(
            args[0] as usize,
            args[1] as usize,
            args[2] as *mut usize,
            args[3] as usize,
        ),
        FSYNC => Ok(0),
        FTRUNCATE64 => {
            syscall_ftruncate64(args[0] as usize, args[1] as usize)
            // 0
        }
        IOCTL => syscall_ioctl(args[0] as usize, args[1] as usize, args[2] as *mut usize),
        // 不做处理即可
        SYNC => Ok(0),
        COPYFILERANGE => syscall_copyfilerange(
            args[0],
            args[1] as *mut usize,
            args[2],
            args[3] as *mut usize,
            args[4],
            args[5],
        ),
        LINKAT => sys_linkat(
            args[0],
            args[1] as *const u8,
            args[2],
            args[3] as *const u8,
            args[4],
        ),
        UNLINKAT => syscall_unlinkat(args[0], args[1] as *const u8, args[2] as usize),
        UTIMENSAT => syscall_utimensat(
            args[0],
            args[1] as *const u8,
            args[2] as *const TimeSecs,
            args[3],
        ),
        EPOLL_CREATE => syscall_epoll_create1(args[0] as usize),
        EPOLL_CTL => syscall_epoll_ctl(
            args[0] as i32,
            args[1] as i32,
            args[2] as i32,
            args[3] as *const EpollEvent,
        ),
        EPOLL_WAIT => syscall_epoll_wait(
            args[0] as i32,
            args[1] as *mut EpollEvent,
            args[2] as i32,
            args[3] as i32,
        ),
        PPOLL => syscall_ppoll(
            args[0] as *mut PollFd,
            args[1] as usize,
            args[2] as *const TimeSecs,
            args[3] as usize,
        ),
        PSELECT6 => syscall_pselect6(
            args[0] as usize,
            args[1] as *mut usize,
            args[2] as *mut usize,
            args[3] as *mut usize,
            args[4] as *const TimeSecs,
            args[5] as usize,
        ),
    }
}
