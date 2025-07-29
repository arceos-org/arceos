use core::{net::SocketAddr, time::Duration};

use axerrno::{LinuxError, LinuxResult};
use axio::{
    PollState, Read, Write,
    buf::{Buf, BufMut},
};
use bitflags::bitflags;
use enum_dispatch::enum_dispatch;

use crate::{
    options::{Configurable, GetSocketOption, SetSocketOption},
    tcp::TcpSocket,
    udp::UdpSocket,
    unix::{UnixSocket, UnixSocketAddr},
};

#[derive(Clone, Debug)]
pub enum SocketAddrEx {
    Ip(SocketAddr),
    Unix(UnixSocketAddr),
}
impl SocketAddrEx {
    pub fn into_ip(self) -> LinuxResult<SocketAddr> {
        match self {
            SocketAddrEx::Ip(addr) => Ok(addr),
            SocketAddrEx::Unix(_) => Err(LinuxError::EAFNOSUPPORT),
        }
    }

    pub fn into_unix(self) -> LinuxResult<UnixSocketAddr> {
        match self {
            SocketAddrEx::Ip(_) => Err(LinuxError::EAFNOSUPPORT),
            SocketAddrEx::Unix(addr) => Ok(addr),
        }
    }
}

bitflags! {
    /// Flags for sending data to a socket.
    ///
    /// See [`SocketOps::send`].
    #[derive(Debug, Clone, Copy)]
    pub struct SendFlags: u32 {
    }
}

bitflags! {
    /// Flags for receiving data from a socket.
    ///
    /// See [`SocketOps::recv`].
    #[derive(Debug, Clone, Copy)]
    pub struct RecvFlags: u32 {
        /// Receive data without removing it from the queue.
        const PEEK = 0x01;
        /// For datagram-like sockets, requires [`SocketOps::recv`] to return
        /// the real size of the datagram, even when it is larger than the
        /// buffer.
        const TRUNCATE = 0x02;
    }
}

/// Options for receiving data from a socket.
///
/// See [`SocketOps::recv`].
#[derive(Debug, Clone)]
pub struct RecvOptions {
    /// Timeout for receiving data in ticks.
    pub timeout: Option<Duration>,
}

/// Kind of shutdown operation to perform on a socket.
#[derive(Debug, Clone, Copy)]
pub enum Shutdown {
    Read,
    Write,
    Both,
}

/// Operations that can be performed on a socket.
#[enum_dispatch]
pub trait SocketOps: Configurable {
    /// Binds an unbound socket to the given address and port.
    fn bind(&self, local_addr: SocketAddrEx) -> LinuxResult<()>;
    /// Connects the socket to a remote address.
    fn connect(&self, remote_addr: SocketAddrEx) -> LinuxResult<()>;

    /// Starts listening on the bound address and port.
    fn listen(&self) -> LinuxResult<()> {
        Err(LinuxError::EOPNOTSUPP)
    }
    /// Accepts a connection on a listening socket, returning a new socket.
    fn accept(&self) -> LinuxResult<Socket> {
        Err(LinuxError::EOPNOTSUPP)
    }

    /// Send data to the socket, optionally to a specific address.
    fn send(
        &self,
        src: &mut impl Buf,
        to: Option<SocketAddrEx>,
        flags: SendFlags,
    ) -> LinuxResult<usize>;
    /// Receive data from the socket.
    fn recv(
        &self,
        dst: &mut impl BufMut,
        from: Option<&mut SocketAddrEx>,
        flags: RecvFlags,
    ) -> LinuxResult<usize>;

    /// Get the local endpoint of the socket.
    fn local_addr(&self) -> LinuxResult<SocketAddrEx>;
    /// Get the remote endpoint of the socket.
    fn peer_addr(&self) -> LinuxResult<SocketAddrEx>;

    /// Poll the socket for readiness.
    fn poll(&self) -> LinuxResult<PollState>;

    /// Shutdown the socket, closing the connection.
    fn shutdown(&self, how: Shutdown) -> LinuxResult<()>;
}

/// Network socket abstraction.
#[enum_dispatch(Configurable, SocketOps)]
pub enum Socket {
    Udp(UdpSocket),
    Tcp(TcpSocket),
    Unix(UnixSocket),
}

impl Read for Socket {
    fn read(&mut self, mut buf: &mut [u8]) -> LinuxResult<usize> {
        self.recv(&mut buf, None, RecvFlags::empty())
    }
}

impl Write for Socket {
    fn write(&mut self, mut buf: &[u8]) -> LinuxResult<usize> {
        self.send(&mut buf, None, SendFlags::empty())
    }

    fn flush(&mut self) -> LinuxResult {
        // TODO(mivik): flush
        Ok(())
    }
}
