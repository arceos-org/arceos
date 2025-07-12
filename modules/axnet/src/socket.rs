use core::{net::SocketAddr, time::Duration};

use axerrno::{LinuxError, LinuxResult};
use axio::{PollState, Read, Write};
use bitflags::bitflags;
use enum_dispatch::enum_dispatch;

use crate::{
    options::{Configurable, GetSocketOption, SetSocketOption},
    tcp::TcpSocket,
    udp::UdpSocket,
};

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
pub struct ShutdownKind {
    /// Whether to close the read side of the socket.
    read: bool,
    /// Whether to close the write side of the socket.
    write: bool,
}
impl Default for ShutdownKind {
    fn default() -> Self {
        Self {
            read: true,
            write: true,
        }
    }
}

/// Operations that can be performed on a socket.
#[enum_dispatch]
pub trait SocketOps: Configurable {
    /// Binds an unbound socket to the given address and port.
    fn bind(&self, local_addr: SocketAddr) -> LinuxResult<()>;
    /// Connects the socket to a remote address.
    fn connect(&self, remote_addr: SocketAddr) -> LinuxResult<()>;

    /// Starts listening on the bound address and port.
    fn listen(&self) -> LinuxResult<()> {
        Err(LinuxError::EOPNOTSUPP)
    }
    /// Accepts a connection on a listening socket, returning a new socket.
    fn accept(&self) -> LinuxResult<Socket> {
        Err(LinuxError::EOPNOTSUPP)
    }

    /// Send data to the socket, optionally to a specific address.
    fn send(&self, buf: &[u8], to: Option<SocketAddr>, flags: SendFlags) -> LinuxResult<usize>;
    /// Receive data from the socket.
    fn recv(
        &self,
        buf: &mut [u8],
        from: Option<&mut SocketAddr>,
        flags: RecvFlags,
    ) -> LinuxResult<usize>;

    /// Get the local endpoint of the socket.
    fn local_addr(&self) -> LinuxResult<SocketAddr>;
    /// Get the remote endpoint of the socket.
    fn peer_addr(&self) -> LinuxResult<SocketAddr>;

    /// Poll the socket for readiness.
    fn poll(&self) -> LinuxResult<PollState>;

    /// Shutdown the socket, closing the connection.
    fn shutdown(&self, kind: ShutdownKind) -> LinuxResult<()>;
}

/// Network socket abstraction.
#[enum_dispatch(Configurable, SocketOps)]
pub enum Socket {
    Udp(UdpSocket),
    Tcp(TcpSocket),
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> LinuxResult<usize> {
        self.recv(buf, None, RecvFlags::empty())
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> LinuxResult<usize> {
        self.send(buf, None, SendFlags::empty())
    }

    fn flush(&mut self) -> LinuxResult {
        // TODO(mivik): flush
        Ok(())
    }
}
