use crate::io;
use axnet;
use axnet::SocketAddr;

/// A UDP socket.
pub struct UdpSocket {
    socket: axnet::UdpSocket,
}

impl UdpSocket {
    /// Creates a UDP socket from the given address.
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        let mut socket = axnet::UdpSocket::new();
        socket.bind(addr)?;
        Ok(Self { socket })
    }

    /// Returns the socket address that this socket was created from.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Receives a message on the socket.
    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.socket.recvfrom(buf)
    }

    /// Sends data on the socket to the given address.
    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        self.socket.sendto(buf, addr)
    }
}
