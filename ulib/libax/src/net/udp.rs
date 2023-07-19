use crate::io;
use axnet::{self, SocketAddr};

/// A UDP socket.
pub struct UdpSocket {
    socket: axnet::UdpSocket,
}

impl UdpSocket {
    /// Creates a UDP socket from the given address.
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        let socket = axnet::UdpSocket::new();
        socket.bind(addr)?;
        Ok(Self { socket })
    }

    /// Returns the socket address that this socket was created from.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Returns the socket address of the remote peer this socket was connected to.
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.socket.peer_addr()
    }

    /// Receives a single datagram message on the socket. On success, returns
    /// the number of bytes read and the origin.
    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf)
    }

    /// Receives a single datagram message on the socket, without removing it from
    /// the queue. On success, returns the number of bytes read and the origin.
    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.socket.peek_from(buf)
    }

    /// Sends data on the socket to the given address. On success, returns the
    /// number of bytes written.
    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        self.socket.send_to(buf, addr)
    }

    /// Connects this UDP socket to a remote address, allowing the `send` and
    /// `recv` to be used to send data and also applies filters to only receive
    /// data from the specified address.
    pub fn connect(&self, addr: SocketAddr) -> io::Result<()> {
        self.socket.connect(addr)
    }

    /// Sends data on the socket to the remote address to which it is connected.
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send(buf)
    }

    /// Receives a single datagram message on the socket from the remote address
    /// to which it is connected. On success, returns the number of bytes read.
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.socket.recv(buf)
    }
}
