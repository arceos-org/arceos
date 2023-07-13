extern crate alloc;

use core::{ptr::copy_nonoverlapping, slice::from_raw_parts};

use alloc::sync::Arc;
use axerrno::{AxError, AxResult};
use axfs::monolithic_fs::{file_io::FileExt, FileIO, FileIOType};
use axio::{Read, Seek, Write};
use axnet::{IpAddr, SocketAddr, TcpSocket, UdpSocket};
use axprocess::process::current_process;
use log::info;
use num_enum::TryFromPrimitive;
use spinlock::SpinNoIrq;

#[derive(TryFromPrimitive)]
#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum Domain {
    AF_UNIX = 1,
    AF_INET = 2,
}

#[derive(TryFromPrimitive)]
#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum SocketType {
    /// Provides sequenced, reliable, two-way, connection-based byte streams.
    /// An out-of-band data transmission mechanism may be supported.
    SOCK_STREAM = 1,
    /// Supports datagrams (connectionless, unreliable messages of a fixed maximum length).
    SOCK_DGRAM = 2,
    /// Provides raw network protocol access.
    SOCK_RAW = 3,
    /// Provides a reliable datagram layer that does not guarantee ordering.
    SOCK_RDM = 4,
    /// Provides a sequenced, reliable, two-way connection-based data
    /// transmission path for datagrams of fixed maximum length;
    /// a consumer is required to read an entire packet with each input system call.
    SOCK_SEQPACKET = 5,
    /// Datagram Congestion Control Protocol socket
    SOCK_DCCP = 6,
    /// Obsolete and should not be used in new programs.
    SOCK_PACKET = 10,
    /// Set O_NONBLOCK flag on the open fd
    SOCK_NONBLOCK = 0x800,
    /// Set FD_CLOEXEC flag on the new fd
    SOCK_CLOEXEC = 0x80000,
}

#[derive(TryFromPrimitive)]
#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum SocketOption {
    SO_RCVTIMEO = 0x1006, // receive timeout
}

/// 包装内部的不同协议 Socket
/// 类似 FileDesc，impl FileIO 后加入fd_list
pub struct Socket {
    #[allow(dead_code)]
    domain: Domain,
    socket_type: SocketType,
    inner: SocketInner,
}

pub enum SocketInner {
    Tcp(TcpSocket),
    Udp(UdpSocket),
}

impl Socket {
    fn new(domain: Domain, socket_type: SocketType) -> Self {
        let inner = match socket_type {
            SocketType::SOCK_STREAM | SocketType::SOCK_SEQPACKET => {
                SocketInner::Tcp(TcpSocket::new())
            }
            SocketType::SOCK_DGRAM => SocketInner::Udp(UdpSocket::new()),
            _ => unimplemented!(),
        };
        Self {
            domain,
            socket_type,
            inner,
        }
    }

    /// Return bound address.
    pub fn name(&self) -> AxResult<SocketAddr> {
        match &self.inner {
            SocketInner::Tcp(s) => s.local_addr(),
            SocketInner::Udp(s) => s.local_addr(),
        }
    }

    pub fn bind(&mut self, addr: SocketAddr) -> AxResult {
        match &mut self.inner {
            SocketInner::Tcp(s) => s.bind(addr),
            SocketInner::Udp(s) => s.bind(addr),
        }
    }

    pub fn is_bound(&self) -> bool {
        match &self.inner {
            SocketInner::Tcp(s) => s.local_addr().is_ok(),
            SocketInner::Udp(s) => s.local_addr().is_ok(),
        }
    }

    pub fn sendto(&self, buf: &[u8], addr: SocketAddr) -> AxResult<usize> {
        match &self.inner {
            SocketInner::Tcp(s) => s.send(buf),
            SocketInner::Udp(s) => s.send_to(buf, addr),
        }
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        match &mut self.inner {
            SocketInner::Tcp(s) => s.read(buf),
            SocketInner::Udp(s) => s.read(buf),
        }
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        match &mut self.inner {
            SocketInner::Tcp(s) => s.write(buf),
            SocketInner::Udp(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> AxResult {
        Err(AxError::Unsupported)
    }
}

impl Seek for Socket {
    fn seek(&mut self, _pos: axio::SeekFrom) -> AxResult<u64> {
        Err(AxError::Unsupported)
    }
}

impl FileExt for Socket {
    fn readable(&self) -> bool {
        match &self.inner {
            SocketInner::Tcp(s) => s.poll().map_or(false, |p| p.readable),
            SocketInner::Udp(s) => s.poll().map_or(false, |p| p.readable),
        }
    }

    fn writable(&self) -> bool {
        match &self.inner {
            SocketInner::Tcp(s) => s.poll().map_or(false, |p| p.writable),
            SocketInner::Udp(s) => s.poll().map_or(false, |p| p.writable),
        }
    }

    fn executable(&self) -> bool {
        false
    }
}

impl FileIO for Socket {
    fn get_type(&self) -> FileIOType {
        FileIOType::Socket
    }
}

pub unsafe fn socket_address_from(addr: *const u8) -> SocketAddr {
    let addr = addr as *const u16;
    let domain = Domain::try_from(*addr as usize).expect("Unsupported Domain (Address Family)");
    match domain {
        Domain::AF_UNIX => unimplemented!(),
        Domain::AF_INET => {
            let port = u16::from_be(*addr.add(1));
            let a = (*(addr.add(2) as *const u32)).to_be_bytes();

            // TODO: not tested! This could be a[3], a[2], a[1], a[0]
            let addr = IpAddr::v4(a[0], a[1], a[2], a[3]);
            SocketAddr { addr, port }
        }
    }
}

/// Only support INET (ipv4)
///
/// ipv4 socket address buffer:
/// socket_domain (address_family) u16
/// port u16 (big endian)
/// addr u32 (big endian)
///
/// TODO: Returns error if buf or buf_len is in invalid memory
pub unsafe fn socket_address_to(addr: SocketAddr, buf: *mut u8, buf_len: *mut usize) -> AxResult {
    let mut tot_len = *buf_len;

    *buf_len = 8;

    // 写入 AF_INET
    if tot_len == 0 {
        return Ok(());
    }
    let domain = (Domain::AF_INET as u16).to_ne_bytes();
    let write_len = tot_len.min(2);
    copy_nonoverlapping(domain.as_ptr(), buf, write_len);
    let buf = buf.add(write_len);
    tot_len -= write_len;

    // 写入 port
    if tot_len == 0 {
        return Ok(());
    }
    let port = &addr.port.to_be_bytes();
    let write_len = tot_len.min(2);
    copy_nonoverlapping(port.as_ptr(), buf, write_len);
    let buf = buf.add(write_len);
    tot_len -= write_len;

    // 写入 address
    if tot_len == 0 {
        return Ok(());
    }
    let address = &addr.addr.as_bytes();
    let write_len = tot_len.min(4);
    copy_nonoverlapping(address.as_ptr(), buf, write_len);

    Ok(())
}

pub fn syscall_socket(domain: usize, s_type: usize, _protocol: usize) -> isize {
    let Ok(domain) = Domain::try_from(domain) else {
        return -1;
    };
    let Ok(s_type) = SocketType::try_from(s_type) else {
        return -1;
    };

    let socket = Socket::new(domain, s_type);
    let curr = current_process();
    let mut inner = curr.inner.lock();

    let Ok(fd) = inner.alloc_fd() else {
        return -1;
    };

    inner.fd_manager.fd_table[fd] = Some(Arc::new(SpinNoIrq::new(socket)));

    fd as isize
}

pub fn syscall_bind(fd: usize, addr: *const u8, _addr_len: usize) -> isize {
    let curr = current_process();
    let inner = curr.inner.lock();

    let Some(Some(file)) = inner.fd_manager.fd_table.get(fd) else {
        // EBADF
        return -1;
    };
    let mut file = file.lock();

    let addr = unsafe { socket_address_from(addr) };

    info!("[bind()] binding socket {} to {:?}", fd, addr);

    let Some(socket) = file.as_any_mut().downcast_mut::<Socket>() else {
        // ENOTSOCK
        return -1;
    };

    socket.bind(addr).map_or(-1, |_| 0)
}

/// NOTE: linux man 中没有说明若socket未bound应返回什么错误
pub fn syscall_get_sock_name(fd: usize, addr: *mut u8, addr_len: *mut usize) -> isize {
    let curr = current_process();
    let inner = curr.inner.lock();

    let Some(Some(file)) = inner.fd_manager.fd_table.get(fd) else {
        // EBADF
        return -1;
    };
    let file = file.lock();

    let Some(socket) = file.as_any().downcast_ref::<Socket>() else {
        // ENOTSOCK
        return -1;
    };

    let Ok(name) = socket.name() else {
        return -1;
    };

    info!("[getsockname()] socket {fd} name: {:?}", name);

    unsafe { socket_address_to(name, addr, addr_len) }.map_or(-1, |_| 0)
}

// TODO: flags
/// Calling sendto() will bind the socket if it's not bound.
pub fn syscall_sendto(
    fd: usize,
    buf: *const u8,
    len: usize,
    _flags: usize,
    addr: *const u8,
    addr_len: usize,
) -> isize {
    let curr = current_process();
    let inner = curr.inner.lock();

    let Some(Some(file)) = inner.fd_manager.fd_table.get(fd) else {
        // EBADF
        return -1;
    };

    let mut file = file.lock();
    let Some(socket) = file.as_any_mut().downcast_mut::<Socket>() else {
        // ENOTSOCK
        return -1;
    };

    let addr = match socket.socket_type {
        SocketType::SOCK_STREAM | SocketType::SOCK_SEQPACKET => {
            if !addr.is_null() || addr_len != 0 {
                // EISCONN
                return -1;
            }
            // TODO: if socket isn't connected, return ENOTCONN

            SocketAddr::new(IpAddr::v4(0, 0, 0, 0), 0)
        }
        SocketType::SOCK_DGRAM => {
            if !socket.is_bound() {
                socket
                    .bind(SocketAddr::new(IpAddr::v4(0, 0, 0, 0), 0))
                    .unwrap();
            }

            unsafe { socket_address_from(addr) }
        }
        _ => unimplemented!(),
    };

    // TODO: check if buffer is safe
    let buf = unsafe { from_raw_parts(buf, len) };

    info!("[sendto()] socket {fd} send to {:?}", addr);

    socket.sendto(buf, addr).map_or(-1, |l| l as isize)
}

/// NOTE: only support socket level options (SOL_SOCKET)
pub fn syscall_set_sock_opt(
    fd: usize,
    level: usize,
    opt_name: usize,
    _opt_value: *const u8,
    _opt_len: usize,
) -> isize {
    // SOL_SOCKET
    if level != 1 {
        unimplemented!();
    }

    let curr = current_process();
    let inner = curr.inner.lock();

    let Some(Some(file)) = inner.fd_manager.fd_table.get(fd) else {
        // EBADF
        return -1;
    };

    let mut file = file.lock();
    let Some(_socket) = file.as_any_mut().downcast_ref::<Socket>() else {
        // ENOTSOCK
        return -1;
    };

    let _option = SocketOption::try_from(opt_name);

    0
}
