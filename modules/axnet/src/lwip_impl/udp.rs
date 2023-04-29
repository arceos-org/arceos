use axerrno::{ax_err, AxResult};

use crate::SocketAddr;

/// A UDP socket that provides POSIX-like APIs.
pub struct UdpSocket {}

impl UdpSocket {
    /// Creates a new UDP socket.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    /// Returns the local address and port, or
    /// [`Err(NotConnected)`](AxError::NotConnected) if not connected.
    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        ax_err!(Unsupported, "LWIP Unsupported")
    }

    /// Binds an unbound socket to the given address and port.
    ///
    /// It's must be called before [`sendto`](Self::sendto) and
    /// [`recvfrom`](Self::recvfrom).
    pub fn bind(&mut self, _addr: SocketAddr) -> AxResult {
        ax_err!(Unsupported, "LWIP Unsupported")
    }

    /// Transmits data in the given buffer to the given address.
    pub fn sendto(&self, _buf: &[u8], _addr: SocketAddr) -> AxResult<usize> {
        ax_err!(Unsupported, "LWIP Unsupported")
    }

    /// Receives data from the socket, stores it in the given buffer.
    pub fn recvfrom(&self, _buf: &mut [u8]) -> AxResult<(usize, SocketAddr)> {
        ax_err!(Unsupported, "LWIP Unsupported")
    }

    /// Close the socket.
    pub fn shutdown(&self) -> AxResult {
        ax_err!(Unsupported, "LWIP Unsupported")
    }

    /// Receives data from the socket, stores it in the given buffer, without removing it from the queue.
    pub fn peekfrom(&self, _buf: &mut [u8]) -> AxResult<(usize, SocketAddr)> {
        ax_err!(Unsupported, "LWIP Unsupported")
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {}
}
