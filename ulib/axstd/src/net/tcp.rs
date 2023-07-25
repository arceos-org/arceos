use super::{SocketAddr, ToSocketAddrs};
use crate::io::{self, prelude::*};

use axnet::TcpSocket;

/// A TCP stream between a local and a remote socket.
pub struct TcpStream(TcpSocket);

/// A TCP socket server, listening for connections.
pub struct TcpListener(TcpSocket);

impl TcpStream {
    /// Opens a TCP connection to a remote host.
    ///
    /// `addr` is an address of the remote host. Anything which implements
    /// [`ToSocketAddrs`] trait can be supplied for the address; see this trait
    /// documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `connect` will be attempted with
    /// each of the addresses until a connection is successful. If none of
    /// the addresses result in a successful connection, the error returned from
    /// the last connection attempt (the last address) is returned.
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
        super::each_addr(addr, |addr: io::Result<&SocketAddr>| {
            let addr = addr?;
            let socket = TcpSocket::new();
            socket.connect(*addr)?;
            Ok(TcpStream(socket))
        })
    }

    /// Returns the socket address of the local half of this TCP connection.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.local_addr()
    }

    /// Returns the socket address of the remote peer of this TCP connection.
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }

    /// Shuts down the connection.
    pub fn shutdown(&self) -> io::Result<()> {
        self.0.shutdown()
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.recv(buf)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.send(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl TcpListener {
    /// Creates a new `TcpListener` which will be bound to the specified
    /// address.
    ///
    /// The returned listener is ready for accepting connections.
    ///
    /// Binding with a port number of 0 will request that the OS assigns a port
    /// to this listener. The port allocated can be queried via the
    /// [`TcpListener::local_addr`] method.
    ///
    /// The address type can be any implementor of [`ToSocketAddrs`] trait. See
    /// its documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `bind` will be attempted with
    /// each of the addresses until one succeeds and returns the listener. If
    /// none of the addresses succeed in creating a listener, the error returned
    /// from the last attempt (the last address) is returned.
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        super::each_addr(addr, |addr: io::Result<&SocketAddr>| {
            let addr = addr?;
            let socket = TcpSocket::new();
            socket.bind(*addr)?;
            socket.listen()?;
            Ok(TcpListener(socket))
        })
    }

    /// Returns the local socket address of this listener.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.local_addr()
    }

    /// Accept a new incoming connection from this listener.
    ///
    /// This function will block the calling thread until a new TCP connection
    /// is established. When established, the corresponding [`TcpStream`] and the
    /// remote peer's address will be returned.
    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let socket = self.0.accept()?;
        let addr = socket.peer_addr()?;
        Ok((TcpStream(socket), addr))
    }
}
