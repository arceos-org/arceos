use core::{
    net::SocketAddr,
    sync::atomic::{AtomicBool, Ordering},
};

use axerrno::{LinuxError, LinuxResult, ax_err, bail};
use axhal::time::current_ticks;
use axio::{PollState, Read, Write};
use axsync::Mutex;
use smoltcp::{
    iface::SocketHandle,
    socket::udp::{self, BindError, RecvError, SendError, UdpMetadata},
    wire::{IpEndpoint, IpListenEndpoint},
};
use spin::RwLock;

use super::{SOCKET_SET, SocketSetWrapper, addr::UNSPECIFIED_ENDPOINT};

/// A UDP socket that provides POSIX-like APIs.
pub struct UdpSocket {
    handle: SocketHandle,
    local_addr: RwLock<Option<IpEndpoint>>,
    peer_addr: RwLock<Option<IpEndpoint>>,
    nonblock: AtomicBool,
    reuse_addr: AtomicBool,
}

impl UdpSocket {
    /// Creates a new UDP socket.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let socket = SocketSetWrapper::new_udp_socket();
        let handle = SOCKET_SET.add(socket);
        Self {
            handle,
            local_addr: RwLock::new(None),
            peer_addr: RwLock::new(None),
            nonblock: AtomicBool::new(false),
            reuse_addr: AtomicBool::new(false),
        }
    }

    /// Returns the local address and port, or
    /// [`Err(ENOTCONN)`](LinuxError::ENOTCONN) if not connected.
    pub fn local_addr(&self) -> LinuxResult<SocketAddr> {
        match self.local_addr.try_read() {
            Some(addr) => addr.map(Into::into).ok_or(LinuxError::ENOTCONN),
            None => Err(LinuxError::ENOTCONN),
        }
    }

    /// Returns the remote address and port, or
    /// [`Err(ENOTCONN)`](LinuxError::ENOTCONN) if not connected.
    pub fn peer_addr(&self) -> LinuxResult<SocketAddr> {
        self.remote_endpoint().map(Into::into)
    }

    /// Returns whether this socket is in nonblocking mode.
    #[inline]
    pub fn is_nonblocking(&self) -> bool {
        self.nonblock.load(Ordering::Acquire)
    }

    /// Moves this UDP socket into or out of nonblocking mode.
    ///
    /// This will result in `recv`, `recv_from`, `send`, and `send_to`
    /// operations becoming nonblocking, i.e., immediately returning from their
    /// calls. If the IO operation is successful, `Ok` is returned and no
    /// further action is required. If the IO operation could not be completed
    /// and needs to be retried, an error with kind
    /// [`Err(EAGAIN)`](LinuxError::EAGAIN) is returned.
    #[inline]
    pub fn set_nonblocking(&self, nonblocking: bool) {
        self.nonblock.store(nonblocking, Ordering::Release);
    }

    /// Set the TTL (time-to-live) option for this socket.
    ///
    /// The TTL is the number of hops that a packet is allowed to live.
    pub fn set_socket_ttl(&self, ttl: u8) {
        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
            socket.set_hop_limit(Some(ttl))
        });
    }

    /// Returns whether this socket is in reuse address mode.
    #[inline]
    pub fn is_reuse_addr(&self) -> bool {
        self.reuse_addr.load(Ordering::Acquire)
    }

    /// Moves this UDP socket into or out of reuse address mode.
    ///
    /// When a socket is bound, the `SO_REUSEADDR` option allows multiple
    /// sockets to be bound to the same address if they are bound to
    /// different local addresses. This option must be set before
    /// calling `bind`.
    #[inline]
    pub fn set_reuse_addr(&self, reuse_addr: bool) {
        self.reuse_addr.store(reuse_addr, Ordering::Release);
    }

    /// Binds an unbound socket to the given address and port.
    ///
    /// It's must be called before [`send_to`](Self::send_to) and
    /// [`recv_from`](Self::recv_from).
    pub fn bind(&self, mut local_addr: SocketAddr) -> LinuxResult {
        let mut self_local_addr = self.local_addr.write();

        if local_addr.port() == 0 {
            local_addr.set_port(get_ephemeral_port()?);
        }
        if self_local_addr.is_some() {
            bail!(EINVAL, "already bound");
        }

        let local_endpoint = IpEndpoint::from(local_addr);
        let endpoint = IpListenEndpoint {
            addr: (!local_endpoint.addr.is_unspecified()).then_some(local_endpoint.addr),
            port: local_endpoint.port,
        };

        if !self.is_reuse_addr() {
            // Check if the address is already in use
            SOCKET_SET.bind_check(local_endpoint.addr, local_endpoint.port)?;
        }

        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
            socket.bind(endpoint).map_err(|e| match e {
                BindError::InvalidState => ax_err!(EINVAL, "already bound"),
                BindError::Unaddressable => ax_err!(ECONNREFUSED, "unaddressable"),
            })
        })?;

        *self_local_addr = Some(local_endpoint);
        info!("UDP socket {}: bound on {}", self.handle, endpoint);
        Ok(())
    }

    /// Sends data on the socket to the given address. On success, returns the
    /// number of bytes written.
    pub fn send_to(&self, buf: &[u8], remote_addr: SocketAddr) -> LinuxResult<usize> {
        if remote_addr.port() == 0 || remote_addr.ip().is_unspecified() {
            bail!(EINVAL, "invalid address");
        }
        self.send_impl(buf, IpEndpoint::from(remote_addr))
    }

    fn handle_recv_result(
        buf: &mut [u8],
        result: Result<(&[u8], UdpMetadata), RecvError>,
        deadline: Option<u64>,
    ) -> LinuxResult<(usize, SocketAddr)> {
        match result {
            Ok((src, meta)) => {
                // TODO(mivik): MSG_TRUNC
                let read = src.len().min(buf.len());
                buf[..read].copy_from_slice(&src[..read]);
                if read < src.len() {
                    warn!("UDP message truncated: {} -> {} bytes", src.len(), read);
                }
                Ok((read, meta.endpoint.into()))
            }
            Err(RecvError::Exhausted) => {
                if deadline.is_some_and(|d| current_ticks() > d) {
                    Err(LinuxError::ETIMEDOUT)
                } else {
                    Err(LinuxError::EAGAIN)
                }
            }
            Err(RecvError::Truncated) => {
                unreachable!("UDP socket recv never returns Err(Truncated)")
            }
        }
    }

    /// Receives a single datagram message on the socket. On success, returns
    /// the number of bytes read and the origin.
    pub fn recv_from(
        &self,
        buf: &mut [u8],
        timeout_ticks: Option<u64>,
    ) -> LinuxResult<(usize, SocketAddr)> {
        // TODO(mivik): check buf is writable?
        let deadline = timeout_ticks.map(|ticks| current_ticks() + ticks);
        self.recv_impl(|socket| Self::handle_recv_result(buf, socket.recv(), deadline))
    }

    /// Receives a single datagram message on the socket, without removing it
    /// from the queue. On success, returns the number of bytes read and the
    /// origin.
    pub fn peek_from(&self, buf: &mut [u8]) -> LinuxResult<(usize, SocketAddr)> {
        self.recv_impl(|socket| {
            Self::handle_recv_result(buf, socket.peek().map(|(data, meta)| (data, *meta)), None)
        })
    }

    /// Connects this UDP socket to a remote address, allowing the `send` and
    /// `recv` to be used to send data and also applies filters to only receive
    /// data from the specified address.
    ///
    /// The local port will be generated automatically if the socket is not
    /// bound. It's must be called before [`send`](Self::send) and
    /// [`recv`](Self::recv).
    pub fn connect(&self, addr: SocketAddr) -> LinuxResult {
        let mut self_peer_addr = self.peer_addr.write();

        if self.local_addr.read().is_none() {
            self.bind(SocketAddr::from(UNSPECIFIED_ENDPOINT))?;
        }

        *self_peer_addr = Some(IpEndpoint::from(addr));
        debug!("UDP socket {}: connected to {}", self.handle, addr);
        Ok(())
    }

    /// Sends data on the socket to the remote address to which it is connected.
    pub fn send(&self, buf: &[u8]) -> LinuxResult<usize> {
        let remote_endpoint = self.remote_endpoint()?;
        self.send_impl(buf, remote_endpoint)
    }

    /// Receives a single datagram message on the socket from the remote address
    /// to which it is connected. On success, returns the number of bytes read.
    pub fn recv(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        let remote_endpoint = self.remote_endpoint()?;
        self.recv_impl(|socket| {
            Self::handle_recv_result(buf, socket.recv(), None).and_then(|(len, meta_addr)| {
                if (remote_endpoint.addr.is_unspecified()
                    && remote_endpoint.addr != meta_addr.ip().into())
                    || (remote_endpoint.port != 0 && remote_endpoint.port != meta_addr.port())
                {
                    bail!(EAGAIN);
                }
                Ok(len)
            })
        })
    }

    /// Close the socket.
    pub fn shutdown(&self) -> LinuxResult {
        SOCKET_SET.poll_interfaces();
        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
            debug!("UDP socket {}: shutting down", self.handle);
            socket.close();
        });
        Ok(())
    }

    /// Whether the socket is readable or writable.
    pub fn poll(&self) -> LinuxResult<PollState> {
        if self.local_addr.read().is_none() {
            return Ok(PollState {
                readable: false,
                writable: false,
            });
        }
        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
            Ok(PollState {
                readable: socket.can_recv(),
                writable: socket.can_send(),
            })
        })
    }
}

/// Private methods
impl UdpSocket {
    fn remote_endpoint(&self) -> LinuxResult<IpEndpoint> {
        match self.peer_addr.try_read() {
            Some(addr) => addr.ok_or(LinuxError::ENOTCONN),
            None => Err(LinuxError::ENOTCONN),
        }
    }

    fn send_impl(&self, buf: &[u8], remote_endpoint: IpEndpoint) -> LinuxResult<usize> {
        if self.local_addr.read().is_none() {
            bail!(ENOTCONN);
        }
        // info!("send to addr: {:?}", remote_endpoint);
        self.block_on(|| {
            SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
                if !socket.is_open() {
                    // not connected
                    bail!(ENOTCONN);
                } else if socket.can_send() {
                    socket
                        .send_slice(buf, remote_endpoint)
                        .map_err(|e| match e {
                            SendError::BufferFull => ax_err!(EAGAIN),
                            SendError::Unaddressable => ax_err!(ECONNREFUSED, "unaddressable"),
                        })?;
                    Ok(buf.len())
                } else {
                    // tx buffer is full
                    Err(LinuxError::EAGAIN)
                }
            })
        })
    }

    fn recv_impl<F, T>(&self, mut op: F) -> LinuxResult<T>
    where
        F: FnMut(&mut udp::Socket) -> LinuxResult<T>,
    {
        if self.local_addr.read().is_none() {
            bail!(ENOTCONN);
        }

        self.block_on(|| {
            SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
                if !socket.is_open() {
                    // not bound
                    bail!(ENOTCONN);
                } else if socket.can_recv() {
                    // data available
                    op(socket)
                } else {
                    // no more data
                    Err(LinuxError::EAGAIN)
                }
            })
        })
    }

    fn block_on<F, T>(&self, mut f: F) -> LinuxResult<T>
    where
        F: FnMut() -> LinuxResult<T>,
    {
        if self.is_nonblocking() {
            f()
        } else {
            loop {
                SOCKET_SET.poll_interfaces();
                match f() {
                    Ok(t) => return Ok(t),
                    Err(LinuxError::EAGAIN) => axtask::yield_now(),
                    Err(e) => return Err(e),
                }
            }
        }
    }

    /// To get the socket and call the given function.
    ///
    /// If the socket is not connected, it will return None.
    ///
    /// Or it will return the result of the given function.
    pub fn with_socket<R>(&self, f: impl FnOnce(&udp::Socket) -> R) -> R {
        SOCKET_SET.with_socket(self.handle, |s| f(s))
    }
}

impl Read for UdpSocket {
    fn read(&mut self, buf: &mut [u8]) -> LinuxResult<usize> {
        self.recv(buf)
    }
}

impl Write for UdpSocket {
    fn write(&mut self, buf: &[u8]) -> LinuxResult<usize> {
        self.send(buf)
    }

    fn flush(&mut self) -> LinuxResult {
        Ok(())
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        self.shutdown().ok();
        SOCKET_SET.remove(self.handle);
    }
}

fn get_ephemeral_port() -> LinuxResult<u16> {
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
