use axerrno::{ax_err, ax_err_type, AxError, AxResult};
use axio::PollState;
use axsync::Mutex;

use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp::{self, ConnectError, RecvError, State};
use smoltcp::wire::IpAddress;

use super::{SocketSetWrapper, ETH0, LISTEN_TABLE, SOCKET_SET};
use crate::SocketAddr;

/// A TCP socket that provides POSIX-like APIs.
///
/// - [`connect`] is for TCP clients.
/// - [`bind`], [`listen`], and [`accept`] are for TCP servers.
/// - Other methods are for both TCP clients and servers.
///
/// [`connect`]: TcpSocket::connect
/// [`bind`]: TcpSocket::bind
/// [`listen`]: TcpSocket::listen
/// [`accept`]: TcpSocket::accept
pub struct TcpSocket {
    handle: Option<SocketHandle>, // `None` if is listening
    local_addr: Option<SocketAddr>,
    peer_addr: Option<SocketAddr>,
    nonblock: bool,
}

impl TcpSocket {
    /// Creates a new TCP socket.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let socket = SocketSetWrapper::new_tcp_socket();
        let handle = Some(SOCKET_SET.add(socket));
        Self {
            handle,
            local_addr: None,
            peer_addr: None,
            nonblock: false,
        }
    }

    /// If `handle` is not [`None`], the socket is used for a client to connect
    /// to a server. Otherwise, the socket is used for a server to listen for
    /// connections.
    const fn is_listening(&self) -> bool {
        self.handle.is_none()
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

    /// Moves this TCP stream into or out of nonblocking mode.
    ///
    /// This will result in `read`, `write`, `recv` and `send` operations
    /// becoming nonblocking, i.e., immediately returning from their calls.
    /// If the IO operation is successful, `Ok` is returned and no further
    /// action is required. If the IO operation could not be completed and needs
    /// to be retried, an error with kind  [`Err(WouldBlock)`](AxError::WouldBlock) is
    /// returned.
    pub fn set_nonblocking(&mut self, nonblocking: bool) {
        self.nonblock = nonblocking;
    }

    /// Connects to the given address and port.
    ///
    /// The local port is generated automatically.
    pub fn connect(&mut self, addr: SocketAddr) -> AxResult {
        let handle = if self.is_listening() {
            return ax_err!(AlreadyExists, "socket connect() failed: already connected");
        } else {
            self.handle.unwrap()
        };

        // TODO: check host unreachable
        let local_port = get_ephemeral_port()?;
        let iface = &ETH0.iface;
        let (local_addr, peer_addr) =
            SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                socket
                    .connect(iface.lock().context(), addr, local_port)
                    .or_else(|e| match e {
                        ConnectError::InvalidState => {
                            ax_err!(AlreadyExists, "socket connect() failed")
                        }
                        ConnectError::Unaddressable => {
                            ax_err!(InvalidInput, "socket connect() failed")
                        }
                    })?;
                Ok((socket.local_endpoint(), socket.remote_endpoint()))
            })?;

        loop {
            SOCKET_SET.poll_interfaces();
            let (state, may_recv) = SOCKET_SET.with_socket::<tcp::Socket, _, _>(handle, |socket| {
                (socket.state(), socket.may_recv())
            });
            if may_recv || state == State::Established {
                self.local_addr = local_addr;
                self.peer_addr = peer_addr;
                return Ok(());
            } else if state == State::SynSent {
                axtask::yield_now();
            } else {
                return ax_err!(ConnectionRefused, "socket connect() failed");
            }
        }
    }

    /// Binds an unbound socket to the given address and port.
    ///
    /// If the given port is 0, it generates one automatically.
    ///
    /// It's must be called before [`listen`](Self::listen) and
    /// [`accept`](Self::accept).
    pub fn bind(&mut self, addr: SocketAddr) -> AxResult {
        if self.local_addr.is_some() {
            return ax_err!(InvalidInput, "socket bind() failed: already bound");
        }

        // TODO: check addr is valid
        let mut addr = addr;
        if addr.port == 0 {
            addr.port = get_ephemeral_port()?;
        }
        self.local_addr = Some(addr);
        Ok(())
    }

    /// Starts listening on the bound address and port.
    ///
    /// It's must be called after [`bind`](Self::bind) and before
    /// [`accept`](Self::accept).
    pub fn listen(&mut self) -> AxResult {
        if self.is_listening() {
            return Ok(()); // already listening
        }

        let local_port = if let Some(local_addr) = self.local_addr {
            local_addr.port
        } else {
            let addr = IpAddress::v4(0, 0, 0, 0);
            let port = get_ephemeral_port()?;
            self.local_addr = Some(SocketAddr::new(addr, port));
            port
        };

        LISTEN_TABLE.listen(local_port)?;
        debug!("socket listening on {}", self.local_addr.unwrap());
        let handle = self.handle.take().unwrap(); // should not connect/send/recv any more
        SOCKET_SET.remove(handle);
        Ok(())
    }

    /// Accepts a new connection.
    ///
    /// This function will block the calling thread until a new TCP connection
    /// is established. When established, a new [`TcpSocket`] is returned.
    ///
    /// It's must be called after [`bind`](Self::bind) and [`listen`](Self::listen).
    pub fn accept(&mut self) -> AxResult<TcpSocket> {
        if !self.is_listening() {
            return ax_err!(InvalidInput, "socket accept() failed: not listen");
        }

        let local_port = self
            .local_addr
            .ok_or_else(|| ax_err_type!(InvalidInput, "socket accept() failed: no address bound"))?
            .port;

        loop {
            SOCKET_SET.poll_interfaces();
            match LISTEN_TABLE.accept(local_port) {
                Ok((handle, peer_addr)) => {
                    debug!("socket accepted a new connection {}", peer_addr.unwrap());
                    return Ok(TcpSocket {
                        handle: Some(handle),
                        local_addr: self.local_addr,
                        peer_addr,
                        nonblock: false,
                    });
                }
                Err(AxError::WouldBlock) => {
                    if self.nonblock {
                        return Err(AxError::WouldBlock);
                    } else {
                        axtask::yield_now()
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Close the connection.
    pub fn shutdown(&self) -> AxResult {
        if let Some(handle) = self.handle {
            // stream
            SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                debug!("socket {}: shutting down", handle);
                socket.close();
            });
        } else {
            // listener
            if let Some(local_addr) = self.local_addr {
                LISTEN_TABLE.unlisten(local_addr.port);
            }
        }
        SOCKET_SET.poll_interfaces();
        Ok(())
    }

    /// Receives data from the socket, stores it in the given buffer.
    pub fn recv(&self, buf: &mut [u8]) -> AxResult<usize> {
        let handle = self
            .handle
            .ok_or_else(|| ax_err_type!(NotConnected, "socket recv() failed"))?;
        loop {
            SOCKET_SET.poll_interfaces();
            match SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                if !socket.is_open() {
                    // not connected
                    ax_err!(NotConnected, "socket recv() failed")
                } else if !socket.may_recv() {
                    // connection closed
                    Ok(0)
                } else if socket.can_recv() {
                    // data available
                    // TODO: use socket.recv(|buf| {...})
                    match socket.recv_slice(buf) {
                        Ok(len) => Ok(len),
                        Err(RecvError::Finished) => Ok(0),
                        Err(_) => ax_err!(ConnectionRefused, "socket recv() failed"),
                    }
                } else {
                    // no more data
                    Err(AxError::WouldBlock)
                }
            }) {
                Ok(n) => {
                    return Ok(n);
                }
                Err(AxError::WouldBlock) => {
                    if self.nonblock {
                        return Err(AxError::WouldBlock);
                    } else {
                        axtask::yield_now()
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Transmits data in the given buffer.
    pub fn send(&self, buf: &[u8]) -> AxResult<usize> {
        let handle = self
            .handle
            .ok_or_else(|| ax_err_type!(NotConnected, "socket send() failed"))?;
        loop {
            SOCKET_SET.poll_interfaces();
            match SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                if !socket.is_open() || !socket.may_send() {
                    // not connected
                    ax_err!(NotConnected, "socket send() failed")
                } else if socket.can_send() {
                    // connected, and the tx buffer is not full
                    // TODO: use socket.send(|buf| {...})
                    let len = socket
                        .send_slice(buf)
                        .map_err(|_| ax_err_type!(ConnectionRefused, "socket send() failed"))?;
                    Ok(len)
                } else {
                    // tx buffer is full
                    Err(AxError::WouldBlock)
                }
            }) {
                Ok(n) => {
                    return Ok(n);
                }
                Err(AxError::WouldBlock) => {
                    if self.nonblock {
                        return Err(AxError::WouldBlock);
                    } else {
                        axtask::yield_now()
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Detect whether the socket needs to receive/can send.
    ///
    /// Return is <need to receive, can send>
    pub fn poll(&self) -> AxResult<PollState> {
        SOCKET_SET.poll_interfaces();
        if let Some(handle) = self.handle {
            // stream
            SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                Ok(PollState {
                    readable: socket.is_open() && socket.can_recv(),
                    writable: socket.is_open() && socket.can_send(),
                })
            })
        } else {
            // listener
            let local_port = self
                .local_addr
                .ok_or_else(|| {
                    ax_err_type!(InvalidInput, "socket poll() failed: no address bound")
                })?
                .port;
            Ok(PollState {
                readable: LISTEN_TABLE.can_accept(local_port)?,
                writable: false,
            })
        }
    }
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        self.shutdown().ok();
        if let Some(handle) = self.handle {
            SOCKET_SET.remove(handle);
        }
    }
}

fn get_ephemeral_port() -> AxResult<u16> {
    const PORT_START: u16 = 0xc000;
    const PORT_END: u16 = 0xffff;
    static CURR: Mutex<u16> = Mutex::new(PORT_START);

    let mut curr = CURR.lock();
    let mut tries = 0;
    // TODO: more robust
    while tries <= PORT_END - PORT_START {
        let port = *curr;
        if *curr == PORT_END {
            *curr = PORT_START;
        } else {
            *curr += 1;
        }
        if LISTEN_TABLE.can_listen(port) {
            return Ok(port);
        }
        tries += 1;
    }
    ax_err!(NoMemory, "no avaliable ports!")
}
