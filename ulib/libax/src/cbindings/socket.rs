use alloc::sync::Arc;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;

use axerrno::{LinuxError, LinuxResult};
use axnet::{resolve_socket_addr, Ipv4Addr, SocketAddr, TcpSocket, UdpSocket};

use super::ctypes;
use super::fd_ops::FileLike;
use super::utils::char_ptr_to_str;
use crate::sync::Mutex;

pub enum Socket {
    Udp(Mutex<UdpSocket>),
    Tcp(Mutex<TcpSocket>),
}

impl Socket {
    fn add_to_fd_table(self) -> LinuxResult<c_int> {
        super::fd_ops::add_file_like(Arc::new(self))
    }

    fn from_fd(fd: c_int) -> LinuxResult<Arc<Self>> {
        let f = super::fd_ops::get_file_like(fd)?;
        f.into_any()
            .downcast::<Self>()
            .map_err(|_| LinuxError::EINVAL)
    }

    fn send(&self, buf: &[u8]) -> LinuxResult<usize> {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.lock().send(buf)?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().send(buf)?),
        }
    }

    fn recv(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.lock().recv_from(buf).map(|e| e.0)?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().recv(buf)?),
        }
    }

    fn bind(&self, addr: SocketAddr) -> LinuxResult {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.lock().bind(addr)?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().bind(addr)?),
        }
    }

    fn connect(&self, addr: SocketAddr) -> LinuxResult {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.lock().connect(addr)?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().connect(addr)?),
        }
    }

    fn sendto(&self, buf: &[u8], addr: SocketAddr) -> LinuxResult<usize> {
        match self {
            // diff: must bind before sendto
            Socket::Udp(udpsocket) => Ok(udpsocket.lock().send_to(buf, addr)?),
            Socket::Tcp(_) => Err(LinuxError::EISCONN),
        }
    }

    fn recvfrom(&self, buf: &mut [u8]) -> LinuxResult<(usize, Option<SocketAddr>)> {
        match self {
            // diff: must bind before recvfrom
            Socket::Udp(udpsocket) => Ok(udpsocket
                .lock()
                .recv_from(buf)
                .map(|res| (res.0, Some(res.1)))?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().recv(buf).map(|res| (res, None))?),
        }
    }

    fn listen(&self) -> LinuxResult {
        match self {
            Socket::Udp(_) => Err(LinuxError::EOPNOTSUPP),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().listen()?),
        }
    }

    fn accept(&self) -> LinuxResult<TcpSocket> {
        match self {
            Socket::Udp(_) => Err(LinuxError::EOPNOTSUPP),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().accept()?),
        }
    }

    fn shutdown(&self) -> LinuxResult {
        match self {
            Socket::Udp(udpsocket) => {
                let udpsocket = udpsocket.lock();
                udpsocket.peer_addr()?;
                udpsocket.shutdown()?;
                Ok(())
            }

            Socket::Tcp(tcpsocket) => {
                let tcpsocket = tcpsocket.lock();
                tcpsocket.peer_addr()?;
                tcpsocket.shutdown()?;
                Ok(())
            }
        }
    }
}

impl FileLike for Socket {
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        self.recv(buf)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        self.send(buf)
    }

    fn stat(&self) -> LinuxResult<ctypes::stat> {
        // not really implemented
        let st_mode = 0o140000 | 0o777u32; // S_IFSOCK | rwxrwxrwx
        Ok(ctypes::stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_blksize: 4096,
            ..Default::default()
        })
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
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
                Socket::Tcp(Mutex::new(TcpSocket::new())).add_to_fd_table()
            }
            (ctypes::AF_INET, ctypes::SOCK_DGRAM, ctypes::IPPROTO_UDP) => {
                Socket::Udp(Mutex::new(UdpSocket::new())).add_to_fd_table()
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
        Socket::from_fd(socket_fd)?.bind(addr)?;
        Ok(0)
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
        Socket::from_fd(socket_fd)?.connect(addr)?;
        Ok(0)
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
    flag: c_int, // currently not used
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
        Socket::from_fd(socket_fd)?.sendto(buf, addr)
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
    flag: c_int, // currently not used
) -> ctypes::ssize_t {
    debug!(
        "ax_sendto <= {} {:#x} {} {}",
        socket_fd, buf_ptr as usize, len, flag
    );
    ax_call_body!(ax_send, {
        if buf_ptr.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let buf = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
        Socket::from_fd(socket_fd)?.send(buf)
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
    flag: c_int, // currently not used
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
        let socket = Socket::from_fd(socket_fd)?;
        let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len) };

        let res = socket.recvfrom(buf)?;
        if let Some(addr) = res.1 {
            unsafe {
                *socket_addr = as_c_sockaddr(&addr);
                *addrlen = size_of::<ctypes::sockaddr>() as u32;
            }
        }
        Ok(res.0)
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
    flag: c_int, // currently not used
) -> ctypes::ssize_t {
    debug!(
        "ax_recv <= {} {:#x} {} {}",
        socket_fd, buf_ptr as usize, len, flag
    );
    ax_call_body!(ax_recv, {
        if buf_ptr.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len) };
        Socket::from_fd(socket_fd)?.recv(buf)
    })
}

/// Listen for connections on a socket
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn ax_listen(
    socket_fd: c_int,
    backlog: c_int, // currently not used
) -> ctypes::ssize_t {
    debug!("ax_listen <= {} {}", socket_fd, backlog);
    ax_call_body!(ax_listen, {
        Socket::from_fd(socket_fd)?.listen()?;
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
        let socket = Socket::from_fd(socket_fd)?;
        let new_socket = socket.accept()?;
        let addr = new_socket.peer_addr()?;
        let new_fd = Socket::add_to_fd_table(Socket::Tcp(Mutex::new(new_socket)))?;
        unsafe {
            *socket_addr = as_c_sockaddr(&addr);
            *socket_len = size_of::<ctypes::sockaddr>() as u32;
        }
        Ok(new_fd)
    })
}

/// Shut down a full-duplex connection.
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn ax_shutdown(
    socket_fd: c_int,
    flag: c_int, // currently not used
) -> ctypes::ssize_t {
    debug!("ax_shutdown <= {} {}", socket_fd, flag);
    ax_call_body!(ax_shutdown, {
        Socket::from_fd(socket_fd)?.shutdown()?;
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
        let res = resolve_socket_addr(name?)?;
        for (i, item) in res.iter().enumerate().take(len) {
            addr_slice[i] = as_c_sockaddr(&(*item, 0).into());
        }
        Ok(if len > res.len() { res.len() } else { len })
    })
}
