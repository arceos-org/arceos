use axerrno::{ax_err, ax_err_type, AxError, AxResult};
use axio::PollState;
use axsync::Mutex;

use smoltcp::iface::SocketHandle;
use smoltcp::socket::udp::{self, BindError, SendError};
use smoltcp::wire::{IpAddress, IpListenEndpoint};

use super::{SocketSetWrapper, ETH0, SOCKET_SET};
use crate::SocketAddr;

const UNSPECIFIED_IP: IpAddress = IpAddress::v4(0, 0, 0, 0);

/// A UDP socket that provides POSIX-like APIs.
pub struct UdpSocket {
    handle: SocketHandle,
    local_addr: Option<SocketAddr>,
    peer_addr: Option<SocketAddr>,
    nonblock: bool,
}

impl UdpSocket {
    /// Creates a new UDP socket.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let socket = SocketSetWrapper::new_udp_socket();
        let handle = SOCKET_SET.add(socket);
        Self {
            handle,
            local_addr: None,
            peer_addr: None,
            nonblock: false,
        }
    }

    /// Returns the local address and port, or
    /// [`Err(NotConnected)`](AxError::NotConnected) if not connected.
    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        self.local_addr.ok_or(AxError::NotConnected)
    }

    /// Returns the remote address and port, or
    /// [`Err(NotConnected)`](AxError::NotConnected) if not connected.
    pub fn peer_addr(&self) -> AxResult<SocketAddr> {
        self.peer_addr.ok_or(AxError::NotConnected)
    }

    /// Moves this UDP socket into or out of nonblocking mode.
    ///
    /// This will result in `recv`, `recv_from`, `send`, and `send_to`
    /// operations becoming nonblocking, i.e., immediately returning from their
    /// calls. If the IO operation is successful, `Ok` is returned and no
    /// further action is required. If the IO operation could not be completed
    /// and needs to be retried, an error with kind
    /// [`Err(WouldBlock)`](AxError::WouldBlock) is returned.
    pub fn set_nonblocking(&mut self, nonblocking: bool) {
        self.nonblock = nonblocking;
    }

    /// Binds an unbound socket to the given address and port.
    ///
    /// It's must be called before [`send_to`](Self::send_to) and
    /// [`recv_from`](Self::recv_from).
    pub fn bind(&mut self, mut local_addr: SocketAddr) -> AxResult {
        if local_addr.addr.is_unspecified() && local_addr.addr != UNSPECIFIED_IP {
            return ax_err!(InvalidInput, "socket bind() failed: invalid addr");
        }
        if local_addr.port == 0 {
            local_addr.port = get_ephemeral_port()?;
        }
        if self.local_addr.is_some() {
            return ax_err!(InvalidInput, "socket bind() failed: already bound");
        }

        let endpoint = IpListenEndpoint {
            addr: (!local_addr.addr.is_unspecified()).then_some(local_addr.addr),
            port: local_addr.port,
        };
        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
            socket.bind(endpoint).or_else(|e| match e {
                BindError::InvalidState => ax_err!(AlreadyExists, "socket bind() failed"),
                BindError::Unaddressable => ax_err!(InvalidInput, "socket bind() failed"),
            })
        })?;
        self.local_addr = Some(local_addr);
        debug!("UDP socket bound on {}", endpoint);
        Ok(())
    }

    /// Transmits data in the given buffer to the given address.
    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> AxResult<usize> {
        self.block_on(|| {
            SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
                if !socket.is_open() {
                    // not bound
                    ax_err!(NotConnected, "socket send() failed")
                } else if socket.can_send() {
                    // TODO: size
                    socket.send_slice(buf, addr).map_err(|e| match e {
                        SendError::BufferFull => AxError::WouldBlock,
                        SendError::Unaddressable => {
                            ax_err_type!(ConnectionRefused, "socket send() failed")
                        }
                    })?;
                    Ok(buf.len())
                } else {
                    // tx buffer is full
                    Err(AxError::WouldBlock)
                }
            })
        })
    }

    fn recv_impl<F, T>(&self, mut op: F, err: &str) -> AxResult<T>
    where
        F: FnMut(&mut udp::Socket) -> AxResult<T>,
    {
        self.block_on(|| {
            SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
                if !socket.is_open() {
                    // not connected
                    ax_err!(NotConnected, err)
                } else if socket.can_recv() {
                    // data available
                    op(socket)
                } else {
                    // no more data
                    Err(AxError::WouldBlock)
                }
            })
        })
    }

    /// Receives data from the socket, stores it in the given buffer.
    pub fn recv_from(&self, buf: &mut [u8]) -> AxResult<(usize, SocketAddr)> {
        self.recv_impl(
            |socket| match socket.recv_slice(buf) {
                Ok((len, meta)) => Ok((len, meta.endpoint)),
                Err(_) => Err(AxError::WouldBlock),
            },
            "socket recv_from() failed",
        )
    }

    /// Connects to the given address and port.
    ///
    /// The local port will be generated automatically if the socket is not bound.
    /// It's must be called before [`send`](Self::send) and
    /// [`recv`](Self::recv).
    pub fn connect(&mut self, addr: SocketAddr) -> AxResult {
        if self.local_addr.is_none() {
            self.bind(SocketAddr::new(
                ETH0.iface
                    .lock()
                    .ipv4_addr()
                    .ok_or_else(|| ax_err_type!(BadAddress, "No IPv4 address"))?
                    .into(),
                0,
            ))?;
        }
        self.peer_addr = Some(addr);
        Ok(())
    }

    /// Transmits data in the given buffer to the remote address to which it is connected.
    pub fn send(&self, buf: &[u8]) -> AxResult<usize> {
        self.send_to(buf, self.peer_addr()?)
    }

    /// Recv data in the given buffer from the remote address to which it is connected.
    pub fn recv(&self, buf: &mut [u8]) -> AxResult<usize> {
        let peeraddr = self.peer_addr()?;
        self.recv_impl(
            |socket| match socket.recv_slice(buf) {
                Ok((len, meta)) => {
                    if meta.endpoint == peeraddr {
                        // filter data from the remote address to which it is connected.
                        Ok(len)
                    } else {
                        Err(AxError::WouldBlock)
                    }
                }
                Err(_) => Err(AxError::WouldBlock),
            },
            "socket recv() failed",
        )
    }

    /// Close the socket.
    pub fn shutdown(&self) -> AxResult {
        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
            debug!("UDP socket {}: shutting down", self.handle);
            socket.close();
        });
        SOCKET_SET.poll_interfaces();
        Ok(())
    }

    /// Receives data from the socket, stores it in the given buffer, without removing it from the queue.
    pub fn peek_from(&self, buf: &mut [u8]) -> AxResult<(usize, SocketAddr)> {
        self.recv_impl(
            |socket| match socket.peek_slice(buf) {
                Ok((len, meta)) => Ok((len, meta.endpoint)),
                Err(_) => Err(AxError::WouldBlock),
            },
            "socket peek_from() failed",
        )
    }

    /// Whether the socket is readable or writable.
    pub fn poll(&self) -> AxResult<PollState> {
        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
            Ok(PollState {
                readable: socket.is_open() && socket.can_recv(),
                writable: socket.is_open() && socket.can_send(),
            })
        })
    }

    fn block_on<F, T>(&self, mut f: F) -> AxResult<T>
    where
        F: FnMut() -> AxResult<T>,
    {
        if self.nonblock {
            f()
        } else {
            loop {
                SOCKET_SET.poll_interfaces();
                match f() {
                    Ok(t) => return Ok(t),
                    Err(AxError::WouldBlock) => axtask::yield_now(),
                    Err(e) => return Err(e),
                }
            }
        }
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        self.shutdown().ok();
        SOCKET_SET.remove(self.handle);
    }
}

fn get_ephemeral_port() -> AxResult<u16> {
    const PORT_START: u16 = 0xc000;
    const PORT_END: u16 = 0xffff;
    static CURR: Mutex<u16> = Mutex::new(PORT_START);
    let mut curr = CURR.lock();

    let port = *curr;
    if *curr == PORT_END {
        *curr = PORT_START;
    } else {
        *curr += 1;
    }
    Ok(port)
}
