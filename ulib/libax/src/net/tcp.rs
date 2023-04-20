use crate::io::{self, prelude::*};

use axnet::{SocketAddr, TcpSocket};

/// A TCP stream between a local and a remote socket.
pub struct TcpStream {
    socket: TcpSocket,
}

/// A TCP socket server, listening for connections.
pub struct TcpListener {
    socket: TcpSocket,
}

impl TcpStream {
    /// Opens a TCP connection to a remote host.
    pub fn connect(addr: SocketAddr) -> io::Result<Self> {
        let mut socket = TcpSocket::new();
        socket.connect(addr)?;
        Ok(Self { socket })
    }

    /// Returns the socket address of the local half of this TCP connection.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Returns the socket address of the remote peer of this TCP connection.
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.socket.peer_addr()
    }

    /// Shuts down the connection.
    pub fn shutdown(&self) -> io::Result {
        self.socket.shutdown()
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.socket.recv(buf)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send(buf)
    }

    fn flush(&mut self) -> io::Result {
        Ok(())
    }
}

impl TcpListener {
    /// Creates a new `TcpListener` which will be bound to the specified
    /// address.
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        let mut socket = TcpSocket::new();
        socket.bind(addr)?;
        socket.listen()?;
        Ok(Self { socket })
    }

    /// Returns the local socket address of this listener.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Accept a new incoming connection from this listener.
    ///
    /// This function will block the calling thread until a new TCP connection
    /// is established. When established, the corresponding [`TcpStream`] and the
    /// remote peer's address will be returned.
    pub fn accept(&mut self) -> io::Result<(TcpStream, SocketAddr)> {
        let socket = self.socket.accept()?;
        let addr = socket.peer_addr()?;
        Ok((TcpStream { socket }, addr))
    }
}
