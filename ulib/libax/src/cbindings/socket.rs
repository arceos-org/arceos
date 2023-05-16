use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;

use axerrno::{AxResult, LinuxError, LinuxResult};
use axnet::{resolve_socket_addr, Ipv4Addr, SocketAddr, TcpSocket, UdpSocket};

use super::ctypes;
use super::fd_table::Filelike;
use super::utils::char_ptr_to_str;
use crate::debug;

pub enum Socket {
    Udp(UdpSocket),
    Tcp(TcpSocket),
}

impl Socket {
    pub fn send(&self, buf: &[u8]) -> LinuxResult<usize> {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.send(buf)?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.send(buf)?),
        }
    }

    pub fn recv(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.recv_from(buf).map(|e| e.0)?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.recv(buf)?),
        }
    }

    fn bind(&mut self, addr: SocketAddr) -> LinuxResult {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.bind(addr)?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.bind(addr)?),
        }
    }

    fn connect(&mut self, addr: SocketAddr) -> LinuxResult {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.connect(addr)?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.connect(addr)?),
        }
    }

    fn sendto(&self, buf: &[u8], addr: SocketAddr) -> LinuxResult<usize> {
        match self {
            // diff: must bind before sendto
            Socket::Udp(udpsocket) => Ok(udpsocket.send_to(buf, addr)?),
            Socket::Tcp(_) => Err(LinuxError::EISCONN),
        }
    }

    fn recvfrom(&self, buf: &mut [u8]) -> LinuxResult<(usize, Option<SocketAddr>)> {
        match self {
            // diff: must bind before recvfrom
            Socket::Udp(udpsocket) => {
                Ok(udpsocket.recv_from(buf).map(|res| (res.0, Some(res.1)))?)
            }
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.recv(buf).map(|res| (res, None))?),
        }
    }

    fn listen(&mut self) -> LinuxResult {
        match self {
            Socket::Udp(_) => Err(LinuxError::EOPNOTSUPP),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.listen()?),
        }
    }

    fn accept(&mut self) -> LinuxResult<TcpSocket> {
        match self {
            Socket::Udp(_) => Err(LinuxError::EOPNOTSUPP),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.accept()?),
        }
    }

    fn shutdown(&self) -> LinuxResult {
        match self {
            Socket::Udp(udpsocket) => {
                udpsocket.peer_addr()?;
                udpsocket.shutdown().unwrap();
                Ok(())
            }

            Socket::Tcp(tcpsocket) => {
                tcpsocket.peer_addr()?;
                tcpsocket.shutdown().unwrap();
                Ok(())
            }
        }
    }
}

fn as_c_sockaddr(addr: &SocketAddr) -> ctypes::sockaddr {
    return unsafe {
        *(&ctypes::sockaddr_in {
            sin_family: ctypes::AF_INET as u16,
            sin_port: addr.port.to_be(),
            sin_addr: ctypes::in_addr {
                s_addr: u32::from_be_bytes(addr.addr.as_bytes().try_into().unwrap()).to_be(),
            },
            sin_zero: [0; 8],
        } as *const _ as *const ctypes::sockaddr)
    };
}

fn from_c_sockaddr(
    addr: *const ctypes::sockaddr,
    addrlen: ctypes::socklen_t,
) -> LinuxResult<SocketAddr> {
    if addr.is_null() {
        return Err(LinuxError::EFAULT);
    }
    if addrlen != size_of::<ctypes::sockaddr>() as u32 {
        return Err(LinuxError::EINVAL);
    }
    let mid = unsafe { *(addr as *const ctypes::sockaddr_in) };
    if mid.sin_family != ctypes::AF_INET as u16 {
        return Err(LinuxError::EINVAL);
    }
    let address = Ipv4Addr::from_bytes(&(u32::from_be(mid.sin_addr.s_addr).to_be_bytes()));
    let port = u16::from_be(mid.sin_port);
    let res: SocketAddr = (address, port).into();
    debug!("    load sockaddr:{:#x} => {:?}", addr as usize, res);
    Ok(res)
}

pub(super) fn stat_socket(_socket: &Socket) -> AxResult<ctypes::stat> {
    // not really implemented
    let st_mode = ((0o14_u32) << 12) | (0o777_u32);
    Ok(ctypes::stat {
        st_ino: 1,
        st_nlink: 1,
        st_mode,
        st_uid: 0,
        st_gid: 0,
        st_size: 0,
        st_blocks: 0,
        st_blksize: 512,
        ..Default::default()
    })
}

/// Create an socket for communication.
///
/// Return the socket file descriptor.
#[no_mangle]
pub unsafe extern "C" fn ax_socket(domain: c_int, socktype: c_int, protocol: c_int) -> c_int {
    debug!("ax_socket <= {} {} {}", domain, socktype, protocol);
    let (domain, socktype, protocol) = (domain as u32, socktype as u32, protocol as u32);
    ax_call_body!(ax_socket, {
        match (domain, socktype, protocol) {
            (ctypes::AF_INET, ctypes::SOCK_STREAM, ctypes::IPPROTO_TCP) => {
                Filelike::from_socket(Socket::Tcp(TcpSocket::new()))
                    .add_to_fd_table()
                    .ok_or(LinuxError::ENFILE)
            }
            (ctypes::AF_INET, ctypes::SOCK_DGRAM, ctypes::IPPROTO_UDP) => {
                Filelike::from_socket(Socket::Udp(UdpSocket::new()))
                    .add_to_fd_table()
                    .ok_or(LinuxError::ENFILE)
            }
            _ => Err(LinuxError::EINVAL),
        }
    })
}

/// Bind a address to a socket.
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn ax_bind(
    socket_fd: c_int,
    socket_addr: *const ctypes::sockaddr,
    addrlen: ctypes::socklen_t,
) -> c_int {
    debug!(
        "ax_bind <= {} {:#x} {}",
        socket_fd, socket_addr as usize, addrlen
    );
    ax_call_body!(ax_bind, {
        let addr = from_c_sockaddr(socket_addr, addrlen)?;
        Filelike::from_fd(socket_fd)?
            .into_socket()?
            .lock()
            .bind(addr)
            .map(|_| 0)
    })
}

/// Connects the socket to the address specified.
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn ax_connect(
    socket_fd: c_int,
    socket_addr: *const ctypes::sockaddr,
    addrlen: ctypes::socklen_t,
) -> c_int {
    debug!(
        "ax_connect <= {} {:#x} {}",
        socket_fd, socket_addr as usize, addrlen
    );
    ax_call_body!(ax_connect, {
        let addr = from_c_sockaddr(socket_addr, addrlen)?;
        Filelike::from_fd(socket_fd)?
            .into_socket()?
            .lock()
            .connect(addr)
            .map(|_| 0)
    })
}

/// Send a message on a socket to the address specified.
///
/// Return the number of bytes sent if success.
#[no_mangle]
pub unsafe extern "C" fn ax_sendto(
    socket_fd: c_int,
    buf_ptr: *const c_void,
    len: ctypes::size_t,
    flag: c_int, //no effect
    socket_addr: *const ctypes::sockaddr,
    addrlen: ctypes::socklen_t,
) -> ctypes::ssize_t {
    debug!(
        "ax_sendto <= {} {:#x} {} {} {:#x} {}",
        socket_fd, buf_ptr as usize, len, flag, socket_addr as usize, addrlen
    );
    ax_call_body!(ax_sendto, {
        if buf_ptr.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let addr = from_c_sockaddr(socket_addr, addrlen)?;
        let buf = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
        Filelike::from_fd(socket_fd)?
            .into_socket()?
            .lock()
            .sendto(buf, addr)
    })
}

/// Send a message on a socket to the address connected.
///
/// Return the number of bytes sent if success.
#[no_mangle]
pub unsafe extern "C" fn ax_send(
    socket_fd: c_int,
    buf_ptr: *const c_void,
    len: ctypes::size_t,
    flag: c_int, //no effect
) -> ctypes::ssize_t {
    debug!(
        "ax_sendto <= {} {:#x} {} {}",
        socket_fd, buf_ptr as usize, len, flag
    );
    ax_call_body!(ax_send, {
        if buf_ptr.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let socket = Filelike::from_fd(socket_fd)?.into_socket()?;
        let buf = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
        let l = socket.lock().send(buf)?;
        Ok(l)
    })
}

/// Receive a message on a socket and get its source address.
///
/// Return the number of bytes received if success.
#[no_mangle]
pub unsafe extern "C" fn ax_recvfrom(
    socket_fd: c_int,
    buf_ptr: *mut c_void,
    len: ctypes::size_t,
    flag: c_int, //no effect
    socket_addr: *mut ctypes::sockaddr,
    addrlen: *mut ctypes::socklen_t,
) -> ctypes::ssize_t {
    debug!(
        "ax_recvfrom <= {} {:#x} {} {} {:#x} {:#x}",
        socket_fd, buf_ptr as usize, len, flag, socket_addr as usize, addrlen as usize
    );
    ax_call_body!(ax_recvfrom, {
        if buf_ptr.is_null() || socket_addr.is_null() || addrlen.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let socket = Filelike::from_fd(socket_fd)?.into_socket()?;
        let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len) };

        let a = socket.lock().recvfrom(buf);
        a.map(|res| {
            if let Some(addr) = res.1 {
                unsafe {
                    *socket_addr = as_c_sockaddr(&addr);
                    *addrlen = size_of::<ctypes::sockaddr>() as u32;
                }
            }
            res.0
        })
    })
}

/// Receive a message on a socket.
///
/// Return the number of bytes received if success.
#[no_mangle]
pub unsafe extern "C" fn ax_recv(
    socket_fd: c_int,
    buf_ptr: *mut c_void,
    len: ctypes::size_t,
    flag: c_int, //no effect
) -> ctypes::ssize_t {
    debug!(
        "ax_recv <= {} {:#x} {} {}",
        socket_fd, buf_ptr as usize, len, flag
    );
    ax_call_body!(ax_recv, {
        if buf_ptr.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let socket = Filelike::from_fd(socket_fd)?.into_socket()?;
        let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len) };
        let a = socket.lock().recv(buf)?;
        Ok(a)
    })
}

/// Listen for connections on a socket
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn ax_listen(
    socket_fd: c_int,
    backlog: c_int, // no effect
) -> ctypes::ssize_t {
    debug!("ax_listen <= {} {}", socket_fd, backlog);
    ax_call_body!(ax_listen, {
        let socket = Filelike::from_fd(socket_fd)?.into_socket()?;
        socket.lock().listen()?;
        Ok(0)
    })
}

/// Accept for connections on a socket
///
/// Return file descriptor for the accepted socket if success.
#[no_mangle]
pub unsafe extern "C" fn ax_accept(
    socket_fd: c_int,
    socket_addr: *mut ctypes::sockaddr,
    socket_len: *mut ctypes::socklen_t,
) -> ctypes::ssize_t {
    debug!(
        "ax_accept <= {} {:#x} {:#x}",
        socket_fd, socket_addr as usize, socket_len as usize
    );
    ax_call_body!(ax_accept, {
        if socket_addr.is_null() || socket_len.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let socket = Filelike::from_fd(socket_fd)?.into_socket()?;
        let ressocket = socket.lock().accept()?;
        let addr = ressocket.peer_addr()?;
        let fd = Filelike::from_socket(Socket::Tcp(ressocket))
            .add_to_fd_table()
            .ok_or(LinuxError::ENFILE)?;
        unsafe {
            *socket_addr = as_c_sockaddr(&addr);
            *socket_len = size_of::<ctypes::sockaddr>() as u32;
        }
        Ok(fd)
    })
}

/// Shut down a full-duplex connection.
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn ax_shutdown(
    socket_fd: c_int,
    flag: c_int, // no effect
) -> ctypes::ssize_t {
    debug!("ax_shutdown <= {} {}", socket_fd, flag);
    ax_call_body!(ax_shutdown, {
        let socket = Filelike::from_fd(socket_fd)?.into_socket()?;
        socket.lock().shutdown()?;
        Ok(0)
    })
}

/// Query addresses for a domain name.
///
/// Return address number if success.
#[no_mangle]
pub unsafe extern "C" fn ax_resolve_sockaddr(
    node: *const c_char,
    addr: *mut ctypes::sockaddr,
    len: ctypes::size_t,
) -> c_int {
    let name = char_ptr_to_str(node);
    debug!(
        "ax_resolve_sockaddr <= {:?} {:#x} {}",
        name, addr as usize, len
    );
    ax_call_body!(ax_resolve_sockaddr, {
        if addr.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let addr_slice = unsafe { core::slice::from_raw_parts_mut(addr, len) };
        let res = resolve_socket_addr(name?).map_err(|_| LinuxError::EINVAL)?;
        for (i, item) in res.iter().enumerate().take(len) {
            addr_slice[i] = as_c_sockaddr(&(*item, 0).into());
        }
        Ok(if len > res.len() { res.len() } else { len })
    })
}
