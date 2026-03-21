use core::{
    ffi::{c_char, c_int, c_void},
    mem::size_of,
};

use arceos_posix_api::ctypes::{
    FIONBIO, IPPROTO_TCP, SO_ERROR, SO_KEEPALIVE, SO_LINGER, SO_RCVBUF, SO_RCVTIMEO, SO_REUSEADDR,
    SO_SNDBUF, SO_SNDTIMEO, SOL_SOCKET, TCP_NODELAY, addrinfo, nfds_t, pollfd, sockaddr, socklen_t,
    timeval,
};
use log::info;

use super::io::set_errno;

#[unsafe(no_mangle)]
pub fn sys_freeaddrinfo(addr_info: *mut addrinfo) {
    info!("[sys_freeaddrinfo] addr_info: {:p}", addr_info);
    unsafe { arceos_posix_api::sys_freeaddrinfo(addr_info) }
}

#[unsafe(no_mangle)]
pub fn sys_getaddrinfo(
    nodename: *const c_char,
    servname: *const c_char,
    hints: *const addrinfo,
    res: *mut *mut addrinfo,
) -> i32 {
    info!(
        "[sys_getaddrinfo] nodename: {:?}, servname: {:?}",
        nodename, servname
    );
    let result = unsafe { arceos_posix_api::sys_getaddrinfo(nodename, servname, hints, res) };
    if result > 0 {
        // hermit expected us to return 0 if success
        0
    } else {
        -1
    }
}

#[unsafe(no_mangle)]
pub fn sys_send(s: i32, mem: *const c_void, len: usize, flags: i32) -> isize {
    info!("[sys_send] socket: {}, len: {}, flags: {}", s, len, flags);
    arceos_posix_api::sys_send(s, mem, len, flags)
}

#[unsafe(no_mangle)]
pub fn sys_socket(domain: i32, type_: i32, protocol: i32) -> i32 {
    info!(
        "[sys_socket] domain: {}, type: {}, protocol: {}",
        domain, type_, protocol
    );
    arceos_posix_api::sys_socket(domain, type_, protocol)
}

#[unsafe(no_mangle)]
pub fn sys_connect(socket_fd: i32, name: *const sockaddr, namelen: socklen_t) -> i32 {
    info!(
        "[sys_connect] socket_fd: {}, namelen: {}",
        socket_fd, namelen
    );
    info!("name: {:?}", unsafe { *name });
    let ret = arceos_posix_api::sys_connect(socket_fd, name, namelen);
    // POSIX: non-blocking connect returns EINPROGRESS, not EAGAIN.
    // ArceOS's TcpSocket::connect() returns WouldBlock (→ EAGAIN) when
    // nonblocking, but hermit std's connect_timeout() expects EINPROGRESS.
    if ret == -(axerrno::LinuxError::EAGAIN.code() as i32) {
        -(axerrno::LinuxError::EINPROGRESS.code() as i32)
    } else {
        ret
    }
}

#[unsafe(no_mangle)]
pub fn sys_recv(socket: i32, buf: *mut u8, len: usize, flags: i32) -> isize {
    info!(
        "[sys_recv] socket: {}, len: {}, flags: {}",
        socket, len, flags
    );
    arceos_posix_api::sys_recv(socket, buf as _, len, flags)
}

#[unsafe(no_mangle)]
pub fn sys_listen(socket_fd: i32, backlog: i32) -> i32 {
    info!(
        "[sys_listen] socket_fd: {}, backlog: {}",
        socket_fd, backlog
    );
    arceos_posix_api::sys_listen(socket_fd, backlog)
}

#[unsafe(no_mangle)]
pub fn sys_bind(socket_fd: i32, name: *const sockaddr, namelen: socklen_t) -> i32 {
    info!("[sys_bind] socket_fd: {}, namelen: {}", socket_fd, namelen);
    arceos_posix_api::sys_bind(socket_fd, name, namelen)
}

#[unsafe(no_mangle)]
pub fn sys_getsockname(socket_fd: i32, name: *mut sockaddr, namelen: *mut socklen_t) -> i32 {
    info!(
        "[sys_getsockname] socket_fd: {}, namelen: {:p}",
        socket_fd, namelen
    );
    unsafe { arceos_posix_api::sys_getsockname(socket_fd, name, namelen) }
}

#[unsafe(no_mangle)]
pub fn sys_accept(socket_fd: i32, addr: *mut sockaddr, addrlen: *mut socklen_t) -> i32 {
    info!(
        "[sys_accept] socket_fd: {}, addrlen: {:p}",
        socket_fd, addrlen
    );
    unsafe { arceos_posix_api::sys_accept(socket_fd, addr, addrlen) }
}

#[unsafe(no_mangle)]
pub fn sys_setsockopt(
    socket_fd: i32,
    level: i32,
    optname: i32,
    optval: *const c_void,
    optlen: socklen_t,
) -> i32 {
    info!(
        "[setsockopt] socket_fd: {}, level: {}, optname: {}, optlen: {}",
        socket_fd, level, optname, optlen
    );
    unsafe { arceos_posix_api::sys_setsockopt(socket_fd, level, optname, optval, optlen) }
}

#[unsafe(no_mangle)]
pub fn sys_getpeername(socket_fd: i32, addr: *mut sockaddr, addrlen: *mut socklen_t) -> i32 {
    info!(
        "[sys_getpeername] socket_fd: {}, addrlen: {:p}",
        socket_fd, addrlen
    );
    unsafe { arceos_posix_api::sys_getpeername(socket_fd, addr, addrlen) }
}

#[unsafe(no_mangle)]
pub fn sys_shutdown(socket_fd: i32, how: i32) -> i32 {
    info!("[sys_shutdown] socket_fd: {}, how: {}", socket_fd, how);
    arceos_posix_api::sys_shutdown(socket_fd, how)
}

/// Set I/O control options on a file descriptor.
///
/// Currently only supports `FIONBIO` (set non-blocking mode), which is
/// used by hermit std's `Socket::set_nonblocking()`.
///
/// Returns 0 on success, negative errno on error.
#[unsafe(no_mangle)]
pub fn sys_ioctl(fd: i32, cmd: i32, argp: *mut c_void) -> i32 {
    info!("[sys_ioctl] fd: {}, cmd: {:#x}", fd, cmd);
    if cmd == FIONBIO {
        if argp.is_null() {
            return -(axerrno::LinuxError::EFAULT.code() as i32);
        }
        let val = unsafe { *(argp as *const c_int) };
        let nonblocking = val != 0;
        info!("[sys_ioctl] FIONBIO: nonblocking={}", nonblocking);
        match arceos_posix_api::sys_fcntl(
            fd,
            arceos_posix_api::ctypes::F_SETFL as c_int,
            if nonblocking {
                arceos_posix_api::ctypes::O_NONBLOCK as usize
            } else {
                0
            },
        ) {
            r if r >= 0 => 0,
            r => r,
        }
    } else {
        info!("[sys_ioctl] unsupported cmd: {:#x}", cmd);
        -(axerrno::LinuxError::EINVAL.code() as i32)
    }
}

/// Poll file descriptors for I/O readiness.
///
/// Hermit std's `connect_timeout()` calls `poll()` with POSIX semantics:
/// returns the number of ready fds on success, 0 on timeout, or **-1** on
/// error (with the actual error code retrievable via `sys_get_errno`).
///
/// The underlying `arceos_posix_api::sys_poll` returns negative errno on
/// error (via `syscall_body!`), so we translate here.
#[unsafe(no_mangle)]
pub fn sys_poll(fds: *mut pollfd, nfds: nfds_t, timeout: i32) -> i32 {
    info!(
        "[sys_poll] fds: {:p}, nfds: {}, timeout: {}ms",
        fds, nfds, timeout
    );
    let ret = arceos_posix_api::sys_poll(fds, nfds, timeout);
    if ret < 0 {
        // Convert from -errno to POSIX -1 + set errno
        set_errno(-ret);
        -1
    } else {
        ret
    }
}

/// Get socket options.
///
/// Used by hermit std for `timeout()` (SO_RCVTIMEO/SO_SNDTIMEO),
/// `take_error()` (SO_ERROR), `nodelay()` (TCP_NODELAY), and
/// `linger()` (SO_LINGER).
///
/// Returns 0 on success, negative errno on error.
#[unsafe(no_mangle)]
pub fn sys_getsockopt(
    socket_fd: i32,
    level: i32,
    optname: i32,
    optval: *mut c_void,
    optlen: *mut socklen_t,
) -> i32 {
    info!(
        "[sys_getsockopt] socket_fd: {}, level: {}, optname: {:#x}",
        socket_fd, level, optname
    );

    if optval.is_null() || optlen.is_null() {
        return -(axerrno::LinuxError::EFAULT.code() as i32);
    }

    // For most options we return sensible defaults since ArceOS's
    // smoltcp-based network stack doesn't track all of these.
    //
    // Note: some hermit ABI constants (IPPROTO_TCP etc.) are u32 while
    // the function parameters are i32, so we use if-else for clarity.
    if level == SOL_SOCKET && optname == SO_ERROR {
        // Return 0 (no pending error).
        // Used by connect_timeout to check connection status.
        let val: c_int = 0;
        write_sockopt(optval, optlen, &val)
    } else if level == SOL_SOCKET && (optname == SO_RCVTIMEO || optname == SO_SNDTIMEO) {
        // Return zero timeval = no timeout set.
        let tv = timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        write_sockopt(optval, optlen, &tv)
    } else if level == SOL_SOCKET && optname == SO_LINGER {
        let val = arceos_posix_api::ctypes::linger {
            l_onoff: 0,
            l_linger: 0,
        };
        write_sockopt(optval, optlen, &val)
    } else if level == SOL_SOCKET && (optname == SO_REUSEADDR || optname == SO_KEEPALIVE) {
        let val: c_int = 0;
        write_sockopt(optval, optlen, &val)
    } else if level == SOL_SOCKET && (optname == SO_SNDBUF || optname == SO_RCVBUF) {
        // Return a reasonable default buffer size
        let val: c_int = 64 * 1024; // 64KB
        write_sockopt(optval, optlen, &val)
    } else if level == IPPROTO_TCP as i32 && optname == TCP_NODELAY {
        // TCP_NODELAY: Nagle's algorithm. Default off (0).
        let val: c_int = 0;
        write_sockopt(optval, optlen, &val)
    } else {
        info!(
            "[sys_getsockopt] unsupported: level={}, optname={:#x}",
            level, optname
        );
        -(axerrno::LinuxError::ENOPROTOOPT.code() as i32)
    }
}

/// Helper: write a value into the getsockopt output buffer.
fn write_sockopt<T: Copy>(optval: *mut c_void, optlen: *mut socklen_t, val: &T) -> i32 {
    let len = size_of::<T>();
    unsafe {
        if (*optlen as usize) < len {
            return -(axerrno::LinuxError::EINVAL.code() as i32);
        }
        core::ptr::copy_nonoverlapping(val as *const T as *const u8, optval as *mut u8, len);
        *optlen = len as socklen_t;
    }
    0
}
