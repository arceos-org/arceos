use core::ffi::c_char;
pub use core::ffi::{c_int, c_short, c_void};

use crate::ctypes_gen;

pub type ssize_t = isize;
pub type size_t = usize;

pub use crate::ctypes_gen::{mode_t, off_t};

/// A thread handle type
pub type Tid = u32;

/// Maximum number of priorities
pub const NO_PRIORITIES: usize = 31;

/// Priority of a thread
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Priority(u8);

impl Priority {
    pub const fn into(self) -> u8 {
        self.0
    }

    pub const fn from(x: u8) -> Self {
        Priority(x)
    }
}

pub const HIGH_PRIO: Priority = Priority::from(3);
pub const NORMAL_PRIO: Priority = Priority::from(2);
pub const LOW_PRIO: Priority = Priority::from(1);

pub const FUTEX_RELATIVE_TIMEOUT: u32 = 1;
pub const CLOCK_REALTIME: clockid_t = 1;
pub const CLOCK_MONOTONIC: clockid_t = 4;
pub const STDIN_FILENO: c_int = 0;
pub const STDOUT_FILENO: c_int = 1;
pub const STDERR_FILENO: c_int = 2;
pub const O_RDONLY: u32 = 0o0;
pub const O_WRONLY: u32 = 0o1;
pub const O_RDWR: u32 = 0o2;
pub const O_CREAT: u32 = 0o100;
pub const O_EXCL: u32 = 0o200;
pub const O_TRUNC: u32 = 0o1000;
pub const O_APPEND: u32 = 0o2000;
pub const O_NONBLOCK: u32 = 0o4000;
pub const O_DIRECTORY: u32 = 0o200000;
pub const O_EXEC: u32 = crate::ctypes_gen::O_EXEC;
pub const F_DUPFD: u32 = 0;
pub const F_GETFD: u32 = 1;
pub const F_SETFD: u32 = 2;
pub const F_GETFL: u32 = 3;
pub const F_SETFL: u32 = 4;
pub const FD_CLOEXEC: i32 = 1;

// will not be used by hermit
pub const F_DUPFD_CLOEXEC: u32 = F_DUPFD;

/// returns true if file descriptor `fd` is a tty
pub fn isatty(_fd: c_int) -> bool {
    false
}

/// `timespec` is used by `clock_gettime` to retrieve the
/// current time
#[derive(Default, Copy, Clone, Debug)]
#[repr(C)]
pub struct timespec {
    /// seconds
    pub tv_sec: time_t,
    /// nanoseconds
    pub tv_nsec: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct timeval {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t,
}

/// The largest number `rand` will return
pub const RAND_MAX: i32 = 2_147_483_647;

// POSIX abi use this
pub const MAXADDRS: u32 = 48;

pub const AF_UNSPEC: u32 = 0;
/// Socket address family: IPv4
pub const AF_INET: u32 = 3;
/// Socket address family: IPv6
pub const AF_INET6: u32 = 1;
/// Socket address family: VSOCK protocol for hypervisor-guest communication
pub const AF_VSOCK: u32 = 2;
pub const IPPROTO_IP: u32 = 0;
pub const IPPROTO_IPV6: u32 = 41;
pub const IPPROTO_UDP: u32 = 17;
pub const IPPROTO_TCP: u32 = 6;
pub const IPV6_ADD_MEMBERSHIP: i32 = 12;
pub const IPV6_DROP_MEMBERSHIP: i32 = 13;
pub const IPV6_MULTICAST_LOOP: i32 = 19;
pub const IPV6_V6ONLY: i32 = 27;
pub const IP_TOS: i32 = 1;
pub const IP_TTL: i32 = 2;
pub const IP_MULTICAST_TTL: i32 = 5;
pub const IP_MULTICAST_LOOP: i32 = 7;
pub const IP_ADD_MEMBERSHIP: i32 = 3;
pub const IP_DROP_MEMBERSHIP: i32 = 4;
pub const SHUT_RD: i32 = 0;
pub const SHUT_WR: i32 = 1;
pub const SHUT_RDWR: i32 = 2;
/// Socket supports datagrams (connectionless,  unreliable  messages of a fixed maximum length)
pub const SOCK_DGRAM: u32 = 2;
/// Socket provides sequenced, reliable,  two-way,  connection-based byte streams.
pub const SOCK_STREAM: u32 = 1;
/// Set the O_NONBLOCK file status flag on the open socket
pub const SOCK_NONBLOCK: u32 = 0o4000;
/// Set  the  close-on-exec flag on the new socket
pub const SOCK_CLOEXEC: u32 = 0o40000;
pub const SOL_SOCKET: i32 = 4095;
pub const SO_REUSEADDR: i32 = 0x0004;
pub const SO_KEEPALIVE: i32 = 0x0008;
pub const SO_BROADCAST: i32 = 0x0020;
pub const SO_LINGER: i32 = 0x0080;
pub const SO_SNDBUF: i32 = 0x1001;
pub const SO_RCVBUF: i32 = 0x1002;
pub const SO_SNDTIMEO: i32 = 0x1005;
pub const SO_RCVTIMEO: i32 = 0x1006;
pub const SO_ERROR: i32 = 0x1007;
pub const TCP_NODELAY: i32 = 1;
pub const MSG_PEEK: i32 = 1;
pub const FIONBIO: i32 = 0x8008667eu32 as i32;
pub const EAI_AGAIN: i32 = 2;
pub const EAI_BADFLAGS: i32 = 3;
pub const EAI_FAIL: i32 = 4;
pub const EAI_FAMILY: i32 = 5;
pub const EAI_MEMORY: i32 = 6;
pub const EAI_NODATA: i32 = 7;
pub const EAI_NONAME: i32 = 8;
pub const EAI_SERVICE: i32 = 9;
pub const EAI_SOCKTYPE: i32 = 10;
pub const EAI_SYSTEM: i32 = 11;
pub const EAI_OVERFLOW: i32 = 14;
pub const POLLIN: i16 = 0x1;
pub const POLLPRI: i16 = 0x2;
pub const POLLOUT: i16 = 0x4;
pub const POLLERR: i16 = 0x8;
pub const POLLHUP: i16 = 0x10;
pub const POLLNVAL: i16 = 0x20;
pub const POLLRDNORM: i16 = 0x040;
pub const POLLRDBAND: i16 = 0x080;
pub const POLLWRNORM: i16 = 0x0100;
pub const POLLWRBAND: i16 = 0x0200;
pub const POLLRDHUP: i16 = 0x2000;
pub const EFD_SEMAPHORE: i16 = 0o1;
pub const EFD_NONBLOCK: i16 = 0o4000;
pub const EFD_CLOEXEC: i16 = 0o40000;
pub const IOV_MAX: usize = 1024;
/// VMADDR_CID_ANY means that any address is possible for binding
pub const VMADDR_CID_ANY: u32 = u32::MAX;
pub const VMADDR_CID_HYPERVISOR: u32 = 0;
pub const VMADDR_CID_LOCAL: u32 = 1;
pub const VMADDR_CID_HOST: u32 = 2;
pub type sa_family_t = u8;
pub type socklen_t = u32;
pub type in_addr_t = u32;
pub type in_port_t = u16;
pub type time_t = i64;
pub type useconds_t = u32;
pub type suseconds_t = i32;
pub type nfds_t = usize;
pub type sem_t = *const c_void;
pub type pid_t = i32;
pub type clockid_t = u32;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct in_addr {
    pub s_addr: in_addr_t,
}

#[repr(C, align(4))]
#[derive(Debug, Copy, Clone, Default)]
pub struct in6_addr {
    pub s6_addr: [u8; 16],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct sockaddr {
    pub sa_len: u8,
    pub sa_family: sa_family_t,
    pub sa_data: [c_char; 14],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct sockaddr_vm {
    pub svm_len: u8,
    pub svm_family: sa_family_t,
    pub svm_reserved1: u16,
    pub svm_port: u32,
    pub svm_cid: u32,
    pub svm_zero: [u8; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct sockaddr_in {
    pub sin_len: u8,
    pub sin_family: sa_family_t,
    pub sin_port: in_port_t,
    pub sin_addr: in_addr,
    pub sin_zero: [c_char; 8],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct sockaddr_in6 {
    pub sin6_len: u8,
    pub sin6_family: sa_family_t,
    pub sin6_port: in_port_t,
    pub sin6_flowinfo: u32,
    pub sin6_addr: in6_addr,
    pub sin6_scope_id: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct addrinfo {
    pub ai_flags: i32,
    pub ai_family: i32,
    pub ai_socktype: i32,
    pub ai_protocol: i32,
    pub ai_addrlen: socklen_t,
    pub ai_canonname: *mut c_char,
    pub ai_addr: *mut sockaddr,
    pub ai_next: *mut addrinfo,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sockaddr_storage {
    pub s2_len: u8,
    pub ss_family: sa_family_t,
    __ss_pad1: [u8; 6],
    __ss_align: i64,
    __ss_pad2: [u8; 112],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct ip_mreq {
    pub imr_multiaddr: in_addr,
    pub imr_interface: in_addr,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct ipv6_mreq {
    pub ipv6mr_multiaddr: in6_addr,
    pub ipv6mr_interface: u32,
}

// fix compile error, hermit does not use them
#[repr(C)]
#[derive(Copy, Clone)]
pub struct aibuf {
    pub ai: addrinfo,
    pub sa: aibuf_sa,
    pub lock: [c_int; 1usize],
    pub slot: c_short,
    pub ref_: c_short,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union aibuf_sa {
    pub sin: sockaddr_in,
    pub sin6: sockaddr_in6,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct linger {
    pub l_onoff: i32,
    pub l_linger: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct pollfd {
    /// file descriptor
    pub fd: i32,
    /// events to look for
    pub events: i16,
    /// events returned
    pub revents: i16,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct stat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_nlink: u64,
    /// access permissions
    pub st_mode: u32,
    /// user id
    pub st_uid: u32,
    /// group id
    pub st_gid: u32,
    /// device id
    pub st_rdev: u64,
    /// size in bytes
    pub st_size: i64,
    /// block size
    pub st_blksize: i64,
    /// size in blocks
    pub st_blocks: i64,
    /// time of last access
    pub st_atim: timespec,
    /// time of last modification
    pub st_mtim: timespec,
    /// time of last status change
    pub st_ctim: timespec,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct dirent64 {
    /// 64-bit inode number
    pub d_ino: u64,
    /// 64-bit offset to next structure
    pub d_off: i64,
    /// Size of this dirent
    pub d_reclen: u16,
    /// File type
    pub d_type: u8,
    /// Filename (null-terminated)
    pub d_name: [c_char; 256],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Describes  a  region  of  memory, beginning at `iov_base` address and with the size of `iov_len` bytes.
pub struct iovec {
    /// Starting address
    pub iov_base: *mut c_void,
    /// Size of the memory pointed to by iov_base.
    pub iov_len: usize,
}

pub const DT_UNKNOWN: u8 = 0;
pub const DT_FIFO: u8 = 1;
pub const DT_CHR: u8 = 2;
pub const DT_DIR: u8 = 4;
pub const DT_BLK: u8 = 6;
pub const DT_REG: u8 = 8;
pub const DT_LNK: u8 = 10;
pub const DT_SOCK: u8 = 12;
pub const DT_WHT: u8 = 14;

pub const S_IFIFO: u32 = 0o1_0000;
pub const S_IFCHR: u32 = 0o2_0000;
pub const S_IFBLK: u32 = 0o6_0000;
pub const S_IFDIR: u32 = 0o4_0000;
pub const S_IFREG: u32 = 0o10_0000;
pub const S_IFLNK: u32 = 0o12_0000;
pub const S_IFSOCK: u32 = 0o14_0000;
pub const S_IFMT: u32 = 0o17_0000;

/// Pages may not be accessed.
pub const PROT_NONE: u32 = 0;
/// Indicates that the memory region should be readable.
pub const PROT_READ: u32 = 1 << 0;
/// Indicates that the memory region should be writable.
pub const PROT_WRITE: u32 = 1 << 1;
/// Indicates that the memory region should be executable.
pub const PROT_EXEC: u32 = 1 << 2;

/// The file offset is set to offset bytes.
pub const SEEK_SET: i32 = 0;
/// The file offset is set to its current location plus offset bytes.
pub const SEEK_CUR: i32 = 1;
/// The file offset is set to the size of the file plus offset bytes.
pub const SEEK_END: i32 = 2;

/// imported ctypes
/// rlimit
pub use ctypes_gen::{rlimit, RLIMIT_DATA, RLIMIT_NOFILE, RLIMIT_STACK};
/// sysconf
pub use ctypes_gen::{
    _SC_AVPHYS_PAGES, _SC_NPROCESSORS_ONLN, _SC_OPEN_MAX, _SC_PAGE_SIZE, _SC_PHYS_PAGES,
};

