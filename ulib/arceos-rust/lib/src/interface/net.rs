use arceos_posix_api::ctypes::{addrinfo, sockaddr, socklen_t};
use core::ffi::{c_char, c_void};
use log::info;

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
    arceos_posix_api::sys_connect(socket_fd, name, namelen)
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
