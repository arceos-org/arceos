use arceos_api::modules::axlog::info;
use arceos_posix_api::ctypes::{addrinfo, sockaddr, socklen_t, AF_INET, AF_INET6};
use core::ffi::{c_char, c_void};

fn convert_family(family: i32) -> i32 {
    match family {
        3 => AF_INET as _,
        1 => AF_INET6 as _,
        other => other,
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HermitAddrInfo {
    pub ai_flags: i32,
    pub ai_family: i32,
    pub ai_socktype: i32,
    pub ai_protocol: i32,
    pub ai_addrlen: socklen_t,
    pub ai_canonname: *mut c_char,
    pub ai_addr: *mut sockaddr,
    pub ai_next: *mut addrinfo,
}

impl From<HermitAddrInfo> for addrinfo {
    fn from(info: HermitAddrInfo) -> Self {
        addrinfo {
            ai_flags: info.ai_flags,
            ai_family: convert_family(info.ai_family),
            ai_socktype: info.ai_socktype,
            ai_protocol: info.ai_protocol,
            ai_addrlen: info.ai_addrlen,
            ai_canonname: info.ai_canonname,
            ai_addr: info.ai_addr,
            ai_next: info.ai_next,
        }
    }
}

impl Into<HermitAddrInfo> for addrinfo {
    fn into(self) -> HermitAddrInfo {
        HermitAddrInfo {
            ai_flags: self.ai_flags,
            ai_family: match self.ai_family as u32 {
                AF_INET => 3,
                AF_INET6 => 1,
                other => other as i32,
            },
            ai_socktype: self.ai_socktype,
            ai_protocol: self.ai_protocol,
            ai_addrlen: self.ai_addrlen,
            ai_canonname: self.ai_canonname,
            ai_addr: self.ai_addr,
            ai_next: self.ai_next,
        }
    }
}

#[unsafe(no_mangle)]
pub fn sys_freeaddrinfo(addr_info: *mut HermitAddrInfo) {
    let mut info = addrinfo::from(unsafe { *addr_info });
    info!("[sys_freeaddrinfo] addr_info: {:?}", info);
    unsafe { arceos_posix_api::sys_freeaddrinfo(&mut info) }
    unsafe { *addr_info = info.into(); }
}

#[unsafe(no_mangle)]
pub fn sys_getaddrinfo(
    nodename: *const c_char,
    servname: *const c_char,
    hints: *const HermitAddrInfo,
    res: *mut *mut HermitAddrInfo,
) -> i32 {
    let hints = if hints.is_null() {
        core::ptr::null()
    } else {
        let h: addrinfo = unsafe { (*hints).into() };
        &h as *const addrinfo
    };
    info!("[sys_getaddrinfo] nodename: {:?}, servname: {:?}", nodename, servname);
    unsafe { arceos_posix_api::sys_getaddrinfo(nodename, servname, hints, res) }
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
    let domain = convert_family(domain);
    arceos_posix_api::sys_socket(domain, type_, protocol)
}

#[unsafe(no_mangle)]
pub fn sys_connect(socket_fd: i32, name: *const sockaddr, namelen: socklen_t) -> i32 {
    info!(
        "[sys_connect] socket_fd: {}, namelen: {}",
        socket_fd, namelen
    );
    info!("name: {:?}", unsafe { *name });
    let mut name_converted = unsafe { *name };
    name_converted.sa_family = convert_family((name_converted.sa_family >> 8) as _) as _;
    arceos_posix_api::sys_connect(socket_fd, &name_converted, namelen)
}

#[unsafe(no_mangle)]
pub fn sys_recv(socket: i32, buf: *mut u8, len: usize, flags: i32) -> isize {
    info!(
        "[sys_recv] socket: {}, len: {}, flags: {}",
        socket, len, flags
    );
    arceos_posix_api::sys_recv(socket, buf as _, len, flags)
}
