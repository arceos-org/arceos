use axerror::AxResult;

use crate::io::{Read, Write};
use crate::{net_impl, SocketAddr};

pub struct TcpStream {
    socket: net_impl::TcpSocket,
}

pub struct TcpListener {
    socket: net_impl::TcpSocket,
}

impl TcpStream {
    pub fn connect(addr: SocketAddr) -> AxResult<Self> {
        let mut socket = net_impl::TcpSocket::new();
        socket.connect(addr)?;
        Ok(Self { socket })
    }

    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        self.socket.local_addr()
    }

    pub fn peer_addr(&self) -> AxResult<SocketAddr> {
        self.socket.peer_addr()
    }

    pub fn shutdown(&self) -> AxResult {
        self.socket.shutdown()
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        self.socket.recv(buf)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        self.socket.send(buf)
    }

    fn flush(&mut self) -> AxResult {
        Ok(())
    }
}

impl TcpListener {
    pub fn bind(addr: SocketAddr) -> AxResult<Self> {
        let mut socket = net_impl::TcpSocket::new();
        socket.bind(addr)?;
        socket.listen()?;
        Ok(Self { socket })
    }

    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        self.socket.local_addr()
    }

    pub fn accept(&mut self) -> AxResult<(TcpStream, SocketAddr)> {
        let socket = self.socket.accept()?;
        let addr = socket.peer_addr()?;
        Ok((TcpStream { socket }, addr))
    }
}
