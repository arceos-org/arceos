use arceos_api::modules::axlog::info;
use arceos_posix_api::ctypes::{addrinfo, sockaddr, socklen_t, AF_INET, AF_INET6};
use core::ffi::{c_char, c_void};

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
    info!("[sys_getaddrinfo] nodename: {:?}, servname: {:?}", nodename, servname);
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
