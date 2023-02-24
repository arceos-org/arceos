use crate::io::{self, prelude::*};

use axnet::{SocketAddr, TcpSocket};

pub struct TcpStream {
    socket: TcpSocket,
}

pub struct TcpListener {
    socket: TcpSocket,
}

impl TcpStream {
    pub fn connect(addr: SocketAddr) -> io::Result<Self> {
        let mut socket = TcpSocket::new();
        socket.connect(addr)?;
        Ok(Self { socket })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.socket.peer_addr()
    }

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
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        let mut socket = TcpSocket::new();
        socket.bind(addr)?;
        socket.listen()?;
        Ok(Self { socket })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    pub fn accept(&mut self) -> io::Result<(TcpStream, SocketAddr)> {
        let socket = self.socket.accept()?;
        let addr = socket.peer_addr()?;
        Ok((TcpStream { socket }, addr))
    }
}
