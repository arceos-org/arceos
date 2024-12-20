use arceos_posix_api::{
    sys_accept, sys_bind, sys_connect, sys_freeaddrinfo, sys_getaddrinfo, sys_getpeername,
    sys_getsockname, sys_listen, sys_recv, sys_recvfrom, sys_send, sys_sendto, sys_shutdown,
    sys_socket,
};
use core::ffi::{c_char, c_int, c_void};

use crate::{ctypes, utils::e};

/// Create an socket for communication.
///
/// Return the socket file descriptor.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn socket(domain: c_int, socktype: c_int, protocol: c_int) -> c_int {
    e(sys_socket(domain, socktype, protocol))
}

/// Bind a address to a socket.
///
/// Return 0 if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bind(
    socket_fd: c_int,
    socket_addr: *const ctypes::sockaddr,
    addrlen: ctypes::socklen_t,
) -> c_int {
    e(sys_bind(socket_fd, socket_addr, addrlen))
}

/// Connects the socket to the address specified.
///
/// Return 0 if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn connect(
    socket_fd: c_int,
    socket_addr: *const ctypes::sockaddr,
    addrlen: ctypes::socklen_t,
) -> c_int {
    e(sys_connect(socket_fd, socket_addr, addrlen))
}

/// Send a message on a socket to the address specified.
///
/// Return the number of bytes sent if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sendto(
    socket_fd: c_int,
    buf_ptr: *const c_void,
    len: ctypes::size_t,
    flag: c_int, // currently not used
    socket_addr: *const ctypes::sockaddr,
    addrlen: ctypes::socklen_t,
) -> ctypes::ssize_t {
    if socket_addr.is_null() && addrlen == 0 {
        return e(sys_send(socket_fd, buf_ptr, len, flag) as _) as _;
    }
    e(sys_sendto(socket_fd, buf_ptr, len, flag, socket_addr, addrlen) as _) as _
}

/// Send a message on a socket to the address connected.
///
/// Return the number of bytes sent if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn send(
    socket_fd: c_int,
    buf_ptr: *const c_void,
    len: ctypes::size_t,
    flag: c_int, // currently not used
) -> ctypes::ssize_t {
    e(sys_send(socket_fd, buf_ptr, len, flag) as _) as _
}

/// Receive a message on a socket and get its source address.
///
/// Return the number of bytes received if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn recvfrom(
    socket_fd: c_int,
    buf_ptr: *mut c_void,
    len: ctypes::size_t,
    flag: c_int, // currently not used
    socket_addr: *mut ctypes::sockaddr,
    addrlen: *mut ctypes::socklen_t,
) -> ctypes::ssize_t {
    if socket_addr.is_null() {
        return e(sys_recv(socket_fd, buf_ptr, len, flag) as _) as _;
    }
    e(sys_recvfrom(socket_fd, buf_ptr, len, flag, socket_addr, addrlen) as _) as _
}

/// Receive a message on a socket.
///
/// Return the number of bytes received if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn recv(
    socket_fd: c_int,
    buf_ptr: *mut c_void,
    len: ctypes::size_t,
    flag: c_int, // currently not used
) -> ctypes::ssize_t {
    e(sys_recv(socket_fd, buf_ptr, len, flag) as _) as _
}

/// Listen for connections on a socket
///
/// Return 0 if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn listen(
    socket_fd: c_int,
    backlog: c_int, // currently not used
) -> c_int {
    e(sys_listen(socket_fd, backlog))
}

/// Accept for connections on a socket
///
/// Return file descriptor for the accepted socket if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept(
    socket_fd: c_int,
    socket_addr: *mut ctypes::sockaddr,
    socket_len: *mut ctypes::socklen_t,
) -> c_int {
    e(sys_accept(socket_fd, socket_addr, socket_len))
}

/// Shut down a full-duplex connection.
///
/// Return 0 if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shutdown(
    socket_fd: c_int,
    flag: c_int, // currently not used
) -> c_int {
    e(sys_shutdown(socket_fd, flag))
}

/// Query addresses for a domain name.
///
/// Return address number if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getaddrinfo(
    nodename: *const c_char,
    servname: *const c_char,
    hints: *const ctypes::addrinfo,
    res: *mut *mut ctypes::addrinfo,
) -> c_int {
    let ret = e(sys_getaddrinfo(nodename, servname, hints, res));
    match ret {
        r if r < 0 => ctypes::EAI_FAIL,
        0 => ctypes::EAI_NONAME,
        _ => 0,
    }
}

/// Free queried `addrinfo` struct
#[unsafe(no_mangle)]
pub unsafe extern "C" fn freeaddrinfo(res: *mut ctypes::addrinfo) {
    sys_freeaddrinfo(res);
}

/// Get current address to which the socket sockfd is bound.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getsockname(
    sock_fd: c_int,
    addr: *mut ctypes::sockaddr,
    addrlen: *mut ctypes::socklen_t,
) -> c_int {
    e(sys_getsockname(sock_fd, addr, addrlen))
}

/// Get peer address to which the socket sockfd is connected.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getpeername(
    sock_fd: c_int,
    addr: *mut ctypes::sockaddr,
    addrlen: *mut ctypes::socklen_t,
) -> c_int {
    e(sys_getpeername(sock_fd, addr, addrlen))
}
