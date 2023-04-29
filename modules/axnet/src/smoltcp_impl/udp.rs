use axerrno::{ax_err, ax_err_type, AxError, AxResult};

use smoltcp::iface::SocketHandle;
use smoltcp::socket::udp::{self, BindError, SendError};

use super::{SocketSetWrapper, SOCKET_SET};
use crate::SocketAddr;

/// A UDP socket that provides POSIX-like APIs.
pub struct UdpSocket {
    handle: Option<SocketHandle>, // `None` if is listening
    local_addr: Option<SocketAddr>,
}

impl UdpSocket {
    /// Creates a new UDP socket.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let socket = SocketSetWrapper::new_udp_socket();
        let handle = Some(SOCKET_SET.add(socket));
        Self {
            handle,
            local_addr: None,
        }
    }

    /// Returns the local address and port, or
    /// [`Err(NotConnected)`](AxError::NotConnected) if not connected.
    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        self.local_addr.ok_or(AxError::NotConnected)
    }

    /// Binds an unbound socket to the given address and port.
    ///
    /// It's must be called before [`sendto`](Self::sendto) and
    /// [`recvfrom`](Self::recvfrom).
    pub fn bind(&mut self, addr: SocketAddr) -> AxResult {
        let handle = self
            .handle
            .ok_or_else(|| ax_err_type!(InvalidInput, "socket bind() failed"))?;
        if self.local_addr.is_some() {
            return ax_err!(InvalidInput, "socket bind() failed: already bound");
        }
        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(handle, |socket| {
            socket.bind(addr).or_else(|e| match e {
                BindError::InvalidState => {
                    ax_err!(AlreadyExists, "socket bind() failed")
                }
                BindError::Unaddressable => {
                    ax_err!(InvalidInput, "socket bind() failed")
                }
            })?;
            Ok(socket.endpoint())
        })?;
        self.local_addr = Some(addr);
        Ok(())
    }

    /// Transmits data in the given buffer to the given address.
    pub fn sendto(&self, buf: &[u8], addr: SocketAddr) -> AxResult<usize> {
        let handle = self
            .handle
            .ok_or_else(|| ax_err_type!(InvalidInput, "socket bind() failed"))?;
        loop {
            SOCKET_SET.poll_interfaces();
            match SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(handle, |socket| {
                if !socket.is_open() {
                    // not connected
                    ax_err!(NotConnected, "socket send() failed")
                } else if socket.can_send() {
                    // TODO: size
                    socket.send_slice(buf, addr).map_err(|e| match e {
                        SendError::BufferFull => AxError::Again,
                        SendError::Unaddressable => {
                            ax_err_type!(ConnectionRefused, "socket send() failed")
                        }
                    })?;
                    Ok(buf.len())
                } else {
                    // tx buffer is full
                    Err(AxError::Again)
                }
            }) {
                Ok(n) => {
                    SOCKET_SET.poll_interfaces();
                    return Ok(n);
                }
                Err(AxError::Again) => axtask::yield_now(),
                Err(e) => return Err(e),
            }
        }
    }

    /// Receives data from the socket, stores it in the given buffer.
    pub fn recvfrom(&self, buf: &mut [u8]) -> AxResult<(usize, SocketAddr)> {
        let handle = self
            .handle
            .ok_or_else(|| ax_err_type!(InvalidInput, "socket recv() failed"))?;
        loop {
            SOCKET_SET.poll_interfaces();
            match SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(handle, |socket| {
                if !socket.is_open() {
                    // not connected
                    ax_err!(NotConnected, "socket recv() failed")
                } else if socket.can_recv() {
                    // data available
                    // TODO: use socket.recv(|buf| {...})
                    match socket.recv_slice(buf) {
                        Ok(x) => Ok(x),
                        Err(_) => Err(AxError::Again),
                    }
                } else {
                    // no more data
                    Err(AxError::Again)
                }
            }) {
                Ok(x) => {
                    SOCKET_SET.poll_interfaces();
                    return Ok(x);
                }
                Err(AxError::Again) => axtask::yield_now(),
                Err(e) => return Err(e),
            }
        }
    }

    /// Close the socket.
    pub fn shutdown(&self) -> AxResult {
        SOCKET_SET.poll_interfaces();
        if let Some(handle) = self.handle {
            // stream
            SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(handle, |socket| {
                debug!("socket {}: shutting down", handle);
                socket.close();
            });
        } else {
            return ax_err!(InvalidInput, "socket shutdown() failed");
        }
        Ok(())
    }

    /// Receives data from the socket, stores it in the given buffer, without removing it from the queue.
    pub fn peekfrom(&self, buf: &mut [u8]) -> AxResult<(usize, SocketAddr)> {
        let handle = self
            .handle
            .ok_or_else(|| ax_err_type!(InvalidInput, "socket recv() failed"))?;
        loop {
            SOCKET_SET.poll_interfaces();
            match SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(handle, |socket| {
                if !socket.is_open() {
                    // not connected
                    ax_err!(NotConnected, "socket recv() failed")
                } else if socket.can_recv() {
                    // data available
                    // TODO: use socket.recv(|buf| {...})
                    match socket.peek_slice(buf) {
                        Ok(x) => Ok((x.0, *x.1)),
                        Err(_) => Err(AxError::Again),
                    }
                } else {
                    // no more data
                    Err(AxError::Again)
                }
            }) {
                Ok(x) => {
                    SOCKET_SET.poll_interfaces();
                    return Ok(x);
                }
                Err(AxError::Again) => axtask::yield_now(),
                Err(e) => return Err(e),
            }
        }
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        self.shutdown().ok();
        if let Some(handle) = self.handle {
            SOCKET_SET.remove(handle);
        }
    }
}
