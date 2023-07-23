use super::{SocketAddr, ToSocketAddrs};
use crate::io;

/// A UDP socket.
pub struct UdpSocket(axnet::UdpSocket);

impl UdpSocket {
    /// Creates a UDP socket from the given address.
    ///
    /// The address type can be any implementor of [`ToSocketAddrs`] trait. See
    /// its documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `bind` will be attempted with
    /// each of the addresses until one succeeds and returns the socket. If none
    /// of the addresses succeed in creating a socket, the error returned from
    /// the last attempt (the last address) is returned.
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<UdpSocket> {
        super::each_addr(addr, |addr: io::Result<&SocketAddr>| {
            let addr = addr?;
            let socket = axnet::UdpSocket::new();
            socket.bind(*addr)?;
            Ok(UdpSocket(socket))
        })
    }

    /// Returns the socket address that this socket was created from.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.local_addr()
    }

    /// Returns the socket address of the remote peer this socket was connected to.
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }

    /// Receives a single datagram message on the socket. On success, returns
    /// the number of bytes read and the origin.
    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.recv_from(buf)
    }

    /// Receives a single datagram message on the socket, without removing it from
    /// the queue. On success, returns the number of bytes read and the origin.
    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.peek_from(buf)
    }

    /// Sends data on the socket to the given address. On success, returns the
    /// number of bytes written.
    ///
    /// Address type can be any implementor of [`ToSocketAddrs`] trait. See its
    /// documentation for concrete examples.
    ///
    /// It is possible for `addr` to yield multiple addresses, but `send_to`
    /// will only send data to the first address yielded by `addr`.
    pub fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> io::Result<usize> {
        match addr.to_socket_addrs()?.next() {
            Some(addr) => self.0.send_to(buf, addr),
            None => axerrno::ax_err!(InvalidInput, "no addresses to send data to"),
        }
    }

    /// Connects this UDP socket to a remote address, allowing the `send` and
    /// `recv` syscalls to be used to send data and also applies filters to only
    /// receive data from the specified address.
    ///
    /// If `addr` yields multiple addresses, `connect` will be attempted with
    /// each of the addresses until the underlying OS function returns no
    /// error. Note that usually, a successful `connect` call does not specify
    /// that there is a remote server listening on the port, rather, such an
    /// error would only be detected after the first send. If the OS returns an
    /// error for each of the specified addresses, the error returned from the
    /// last connection attempt (the last address) is returned.
    pub fn connect(&self, addr: SocketAddr) -> io::Result<()> {
        super::each_addr(addr, |addr: io::Result<&SocketAddr>| {
            let addr = addr?;
            self.0.connect(*addr)
        })
    }

    /// Sends data on the socket to the remote address to which it is connected.
    ///
    /// [`UdpSocket::connect`] will connect this socket to a remote address. This
    /// method will fail if the socket is not connected.
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.send(buf)
    }

    /// Receives a single datagram message on the socket from the remote address to
    /// which it is connected. On success, returns the number of bytes read.
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.recv(buf)
    }
}
