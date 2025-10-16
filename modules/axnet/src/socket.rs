use alloc::{boxed::Box, vec::Vec};
use core::{
    any::Any,
    fmt::{self, Debug},
    net::SocketAddr,
    task::Context,
};

use axerrno::{AxError, AxResult, LinuxError};
use axio::{Buf, BufMut};
use axpoll::{IoEvents, Pollable};
use bitflags::bitflags;
use enum_dispatch::enum_dispatch;

use crate::{
    options::{Configurable, GetSocketOption, SetSocketOption},
    tcp::TcpSocket,
    udp::UdpSocket,
    unix::{UnixSocket, UnixSocketAddr},
};

#[cfg(feature = "vsock")]
use crate::vsock::{VsockSocket, VsockAddr};

#[derive(Clone, Debug)]
pub enum SocketAddrEx {
    Ip(SocketAddr),
    Unix(UnixSocketAddr),
    #[cfg(feature = "vsock")]
    Vsock(VsockAddr),
}

impl SocketAddrEx {
    pub fn into_ip(self) -> AxResult<SocketAddr> {
        match self {
            SocketAddrEx::Ip(addr) => Ok(addr),
            SocketAddrEx::Unix(_) => Err(AxError::Other(LinuxError::EAFNOSUPPORT)),
            #[cfg(feature = "vsock")]
            SocketAddrEx::Vsock(_) => Err(AxError::Other(LinuxError::EAFNOSUPPORT)),
        }
    }

    pub fn into_unix(self) -> AxResult<UnixSocketAddr> {
        match self {
            SocketAddrEx::Unix(addr) => Ok(addr),
            SocketAddrEx::Ip(_) => Err(AxError::Other(LinuxError::EAFNOSUPPORT)),
            #[cfg(feature = "vsock")]
            SocketAddrEx::Vsock(_) => Err(AxError::Other(LinuxError::EAFNOSUPPORT)),
        }
    }

    #[cfg(feature = "vsock")]
    pub fn into_vsock(self) -> AxResult<VsockAddr> {
        match self {
            SocketAddrEx::Ip(_) => Err(AxError::Other(LinuxError::EAFNOSUPPORT)),
            SocketAddrEx::Unix(_) => Err(AxError::Other(LinuxError::EAFNOSUPPORT)),
            SocketAddrEx::Vsock(addr) => Ok(addr),
        }
    }
}

bitflags! {
    /// Flags for sending data to a socket.
    ///
    /// See [`SocketOps::send`].
    #[derive(Default, Debug, Clone, Copy)]
    pub struct SendFlags: u32 {
    }
}

bitflags! {
    /// Flags for receiving data from a socket.
    ///
    /// See [`SocketOps::recv`].
    #[derive(Default, Debug, Clone, Copy)]
    pub struct RecvFlags: u32 {
        /// Receive data without removing it from the queue.
        const PEEK = 0x01;
        /// For datagram-like sockets, requires [`SocketOps::recv`] to return
        /// the real size of the datagram, even when it is larger than the
        /// buffer.
        const TRUNCATE = 0x02;
    }
}

pub type CMsgData = Box<dyn Any + Send + Sync>;

/// Options for sending data to a socket.
///
/// See [`SocketOps::send`].
#[derive(Default, Debug)]
pub struct SendOptions {
    pub to: Option<SocketAddrEx>,
    pub flags: SendFlags,
    pub cmsg: Vec<CMsgData>,
}

/// Options for receiving data from a socket.
///
/// See [`SocketOps::recv`].
#[derive(Default)]
pub struct RecvOptions<'a> {
    pub from: Option<&'a mut SocketAddrEx>,
    pub flags: RecvFlags,
    pub cmsg: Option<&'a mut Vec<CMsgData>>,
}
impl Debug for RecvOptions<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecvOptions")
            .field("from", &self.from)
            .field("flags", &self.flags)
            .finish()
    }
}

/// Kind of shutdown operation to perform on a socket.
#[derive(Debug, Clone, Copy)]
pub enum Shutdown {
    Read,
    Write,
    Both,
}
impl Shutdown {
    pub fn has_read(&self) -> bool {
        matches!(self, Shutdown::Read | Shutdown::Both)
    }

    pub fn has_write(&self) -> bool {
        matches!(self, Shutdown::Write | Shutdown::Both)
    }
}

/// Operations that can be performed on a socket.
#[enum_dispatch]
pub trait SocketOps: Configurable {
    /// Binds an unbound socket to the given address and port.
    fn bind(&self, local_addr: SocketAddrEx) -> AxResult;
    /// Connects the socket to a remote address.
    fn connect(&self, remote_addr: SocketAddrEx) -> AxResult;

    /// Starts listening on the bound address and port.
    fn listen(&self) -> AxResult {
        Err(AxError::OperationNotSupported)
    }
    /// Accepts a connection on a listening socket, returning a new socket.
    fn accept(&self) -> AxResult<Socket> {
        Err(AxError::OperationNotSupported)
    }

    /// Send data to the socket, optionally to a specific address.
    fn send(&self, src: &mut impl Buf, options: SendOptions) -> AxResult<usize>;
    /// Receive data from the socket.
    fn recv(&self, dst: &mut impl BufMut, options: RecvOptions<'_>) -> AxResult<usize>;

    /// Get the local endpoint of the socket.
    fn local_addr(&self) -> AxResult<SocketAddrEx>;
    /// Get the remote endpoint of the socket.
    fn peer_addr(&self) -> AxResult<SocketAddrEx>;

    /// Shutdown the socket, closing the connection.
    fn shutdown(&self, how: Shutdown) -> AxResult;
}

/// Network socket abstraction.
#[enum_dispatch(Configurable, SocketOps)]
pub enum Socket {
    Udp(UdpSocket),
    Tcp(TcpSocket),
    Unix(UnixSocket),
    #[cfg(feature = "vsock")]
    Vsock(VsockSocket),
}

impl Pollable for Socket {
    fn poll(&self) -> IoEvents {
        match self {
            Socket::Tcp(tcp) => tcp.poll(),
            Socket::Udp(udp) => udp.poll(),
            Socket::Unix(unix) => unix.poll(),
            #[cfg(feature = "vsock")]
            Socket::Vsock(vsock) => vsock.poll(),
        }
    }

    fn register(&self, context: &mut Context<'_>, events: IoEvents) {
        match self {
            Socket::Tcp(tcp) => tcp.register(context, events),
            Socket::Udp(udp) => udp.register(context, events),
            Socket::Unix(unix) => unix.register(context, events),
            #[cfg(feature = "vsock")]
            Socket::Vsock(vsock) => vsock.register(context, events),
        }
    }
}
