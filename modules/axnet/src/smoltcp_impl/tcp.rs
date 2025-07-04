use core::{
    cell::UnsafeCell,
    net::SocketAddr,
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
};

use axerrno::{LinuxError, LinuxResult, ax_err, bail};
use axhal::time::current_ticks;
use axio::{PollState, Read, Write};
use axsync::Mutex;
use axtask::yield_now;
use smoltcp::{
    iface::SocketHandle,
    socket::tcp::{self, ConnectError, State},
    wire::{IpAddress, IpEndpoint, IpListenEndpoint},
};

use super::{ETH0, LISTEN_TABLE, SOCKET_SET, SocketSetWrapper, addr::UNSPECIFIED_ENDPOINT};

// State transitions:
// CLOSED -(connect)-> BUSY -> CONNECTING -> CONNECTED -(shutdown)-> BUSY ->
// CLOSED       |
//       |-(listen)-> BUSY -> LISTENING -(shutdown)-> BUSY -> CLOSED
//       |
//        -(bind)-> BUSY -> CLOSED
const STATE_CLOSED: u8 = 0;
const STATE_BUSY: u8 = 1;
const STATE_CONNECTING: u8 = 2;
const STATE_CONNECTED: u8 = 3;
const STATE_LISTENING: u8 = 4;

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
    state: AtomicU8,
    handle: UnsafeCell<Option<SocketHandle>>,
    local_addr: UnsafeCell<IpEndpoint>,
    peer_addr: UnsafeCell<IpEndpoint>,
    nonblock: AtomicBool,
    reuse_addr: AtomicBool,
}

unsafe impl Sync for TcpSocket {}

impl TcpSocket {
    /// Creates a new TCP socket.
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(STATE_CLOSED),
            handle: UnsafeCell::new(None),
            local_addr: UnsafeCell::new(UNSPECIFIED_ENDPOINT),
            peer_addr: UnsafeCell::new(UNSPECIFIED_ENDPOINT),
            nonblock: AtomicBool::new(false),
            reuse_addr: AtomicBool::new(false),
        }
    }

    /// Creates a new TCP socket that is already connected.
    const fn new_connected(
        handle: SocketHandle,
        local_addr: IpEndpoint,
        peer_addr: IpEndpoint,
    ) -> Self {
        Self {
            state: AtomicU8::new(STATE_CONNECTED),
            handle: UnsafeCell::new(Some(handle)),
            local_addr: UnsafeCell::new(local_addr),
            peer_addr: UnsafeCell::new(peer_addr),
            nonblock: AtomicBool::new(false),
            reuse_addr: AtomicBool::new(false),
        }
    }

    /// Returns the local address and port, or
    /// [`Err(NotConnected)`](LinuxError::NotConnected) if not connected.
    #[inline]
    pub fn local_addr(&self) -> LinuxResult<SocketAddr> {
        // 为了通过测例，已经`bind`但未`listen`的socket也可以返回地址
        match self.get_state() {
            STATE_CONNECTED | STATE_LISTENING | STATE_CLOSED => {
                Ok(SocketAddr::from(unsafe { self.local_addr.get().read() }))
            }
            _ => Err(LinuxError::ENOTCONN),
        }
    }

    /// Returns the remote address and port, or
    /// [`Err(NotConnected)`](LinuxError::NotConnected) if not connected.
    #[inline]
    pub fn peer_addr(&self) -> LinuxResult<SocketAddr> {
        match self.get_state() {
            STATE_CONNECTED | STATE_LISTENING => {
                Ok(SocketAddr::from(unsafe { self.peer_addr.get().read() }))
            }
            _ => Err(LinuxError::ENOTCONN),
        }
    }

    /// Returns whether this socket is in nonblocking mode.
    #[inline]
    pub fn is_nonblocking(&self) -> bool {
        self.nonblock.load(Ordering::Acquire)
    }

    /// Moves this TCP stream into or out of nonblocking mode.
    ///
    /// This will result in `read`, `write`, `recv` and `send` operations
    /// becoming nonblocking, i.e., immediately returning from their calls.
    /// If the IO operation is successful, `Ok` is returned and no further
    /// action is required. If the IO operation could not be completed and needs
    /// to be retried, an error with kind  [`Err(EAGAIN)`](LinuxError::EAGAIN)
    /// is returned.
    #[inline]
    pub fn set_nonblocking(&self, nonblocking: bool) {
        self.nonblock.store(nonblocking, Ordering::Release);
    }

    /// Returns whether this socket is in reuse address mode.
    #[inline]
    pub fn is_reuse_addr(&self) -> bool {
        self.reuse_addr.load(Ordering::Acquire)
    }

    /// Moves this TCP socket into or out of reuse address mode.
    ///
    /// When a socket is bound, the `SO_REUSEADDR` option allows multiple
    /// sockets to be bound to the same address if they are bound to
    /// different local addresses. This option must be set before
    /// calling `bind`.
    #[inline]
    pub fn set_reuse_addr(&self, reuse_addr: bool) {
        self.reuse_addr.store(reuse_addr, Ordering::Release);
    }

    /// Connects to the given address and port.
    ///
    /// The local port is generated automatically.
    pub fn connect(&self, remote_addr: SocketAddr) -> LinuxResult {
        self.update_state(STATE_CLOSED, STATE_CONNECTING, || {
            // SAFETY: no other threads can read or write these fields.
            let handle = unsafe { self.handle.get().read() }
                .unwrap_or_else(|| SOCKET_SET.add(SocketSetWrapper::new_tcp_socket()));

            // // TODO: check remote addr unreachable
            // let (bound_endpoint, remote_endpoint) = self.get_endpoint_pair(remote_addr)?;
            let remote_endpoint = IpEndpoint::from(remote_addr);
            let bound_endpoint = self.bound_endpoint()?;
            info!(
                "TCP connection from {} to {}",
                bound_endpoint, remote_endpoint
            );

            warn!("Temporarily net bridge used");
            let iface = if match remote_endpoint.addr {
                IpAddress::Ipv4(addr) => addr.is_loopback(),
                IpAddress::Ipv6(addr) => addr.is_loopback(),
            } {
                super::LOOPBACK.get().unwrap()
            } else {
                info!("Use eth net");
                &ETH0.iface
            };

            let (local_endpoint, remote_endpoint) = SOCKET_SET
                .with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                    socket
                        .connect(iface.lock().context(), remote_endpoint, bound_endpoint)
                        .map_err(|e| match e {
                            ConnectError::InvalidState => ax_err!(EISCONN, "already conncted"),
                            ConnectError::Unaddressable => ax_err!(ECONNREFUSED, "unaddressable"),
                        })?;
                    Ok::<(IpEndpoint, IpEndpoint), LinuxError>((
                        socket.local_endpoint().unwrap(),
                        socket.remote_endpoint().unwrap(),
                    ))
                })?;
            unsafe {
                // SAFETY: no other threads can read or write these fields as we
                // have changed the state to `BUSY`.
                self.local_addr.get().write(local_endpoint);
                self.peer_addr.get().write(remote_endpoint);
                self.handle.get().write(Some(handle));
            }
            Ok(())
        })
        .map_err(|_| ax_err!(EISCONN, "already conncted"))??;

        // HACK: yield() to let server to listen
        yield_now();

        // Here our state must be `CONNECTING`, and only one thread can run here.
        if self.is_nonblocking() {
            Err(LinuxError::EAGAIN)
        } else {
            self.block_on(|| {
                let PollState { writable, .. } = self.poll_connect()?;
                if !writable {
                    Err(LinuxError::EAGAIN)
                } else if self.get_state() == STATE_CONNECTED {
                    Ok(())
                } else {
                    bail!(ECONNREFUSED, "connection refused");
                }
            })
        }
    }

    /// Binds an unbound socket to the given address and port.
    ///
    /// If the given port is 0, it generates one automatically.
    ///
    /// It's must be called before [`listen`](Self::listen) and
    /// [`accept`](Self::accept).
    pub fn bind(&self, mut local_addr: SocketAddr) -> LinuxResult {
        self.update_state(STATE_CLOSED, STATE_CLOSED, || {
            // TODO: check addr is available
            if local_addr.port() == 0 {
                local_addr.set_port(get_ephemeral_port()?);
            }
            // SAFETY: no other threads can read or write `self.local_addr` as we
            // have changed the state to `BUSY`.
            unsafe {
                let old = self.local_addr.get().read();
                if old != UNSPECIFIED_ENDPOINT {
                    return Err(LinuxError::EINVAL);
                }
                self.local_addr.get().write(IpEndpoint::from(local_addr));
            }
            let local_endpoint = IpEndpoint::from(local_addr);
            let bound_endpoint = self.bound_endpoint()?;
            let handle = unsafe { self.handle.get().read() }
                .unwrap_or_else(|| SOCKET_SET.add(SocketSetWrapper::new_tcp_socket()));
            SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                socket.set_bound_endpoint(bound_endpoint);
            });

            if !self.is_reuse_addr() {
                SOCKET_SET.bind_check(local_endpoint.addr, local_endpoint.port)?;
            }
            Ok(())
        })
        .map_err(|_| ax_err!(EINVAL, "already bound"))?
    }

    /// Starts listening on the bound address and port.
    ///
    /// It's must be called after [`bind`](Self::bind) and before
    /// [`accept`](Self::accept).
    pub fn listen(&self) -> LinuxResult {
        self.update_state(STATE_CLOSED, STATE_LISTENING, || {
            let bound_endpoint = self.bound_endpoint()?;
            unsafe {
                (*self.local_addr.get()).port = bound_endpoint.port;
            }
            LISTEN_TABLE.listen(bound_endpoint)?;
            debug!("TCP socket listening on {}", bound_endpoint);
            Ok(())
        })
        .unwrap_or(Ok(())) // ignore simultaneous `listen`s.
    }

    /// Accepts a new connection.
    ///
    /// This function will block the calling thread until a new TCP connection
    /// is established. When established, a new [`TcpSocket`] is returned.
    ///
    /// It's must be called after [`bind`](Self::bind) and
    /// [`listen`](Self::listen).
    pub fn accept(&self) -> LinuxResult<TcpSocket> {
        if !self.is_listening() {
            bail!(EINVAL, "not listening");
        }

        // SAFETY: `self.local_addr` should be initialized after `bind()`.
        let local_port = unsafe { self.local_addr.get().read().port };
        self.block_on(|| {
            let (handle, (local_addr, peer_addr)) = LISTEN_TABLE.accept(local_port)?;
            info!(
                "TCP socket {}: accepted connection from {}",
                handle, peer_addr
            );
            Ok(TcpSocket::new_connected(handle, local_addr, peer_addr))
        })
    }

    /// Close the connection.
    pub fn shutdown(&self) -> LinuxResult {
        // stream
        self.update_state(STATE_CONNECTED, STATE_CLOSED, || {
            // SAFETY: `self.handle` should be initialized in a connected socket, and
            // no other threads can read or write it.
            let handle = unsafe { self.handle.get().read().unwrap() };
            SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                debug!("TCP socket {}: shutting down", handle);
                socket.close();
            });
            unsafe { self.local_addr.get().write(UNSPECIFIED_ENDPOINT) }; // clear bound address
            SOCKET_SET.poll_interfaces();
            Ok(())
        })
        .unwrap_or(Ok(()))?;

        // listener
        self.update_state(STATE_LISTENING, STATE_CLOSED, || {
            // SAFETY: `self.local_addr` should be initialized in a listening socket,
            // and no other threads can read or write it.
            let local_port = unsafe { self.local_addr.get().read().port };
            unsafe { self.local_addr.get().write(UNSPECIFIED_ENDPOINT) }; // clear bound address
            LISTEN_TABLE.unlisten(local_port);
            SOCKET_SET.poll_interfaces();
            Ok(())
        })
        .unwrap_or(Ok(()))?;

        // ignore for other states
        Ok(())
    }

    /// Close the transmit half of the tcp socket.
    /// It will call `close()` on smoltcp::socket::tcp::Socket. It should send
    /// FIN to remote half.
    ///
    /// This function is for shutdown(fd, SHUT_WR) syscall.
    ///
    /// It won't change TCP state.
    /// It won't affect unconnected sockets (listener).
    pub fn close(&self) {
        let handle = match unsafe { self.handle.get().read() } {
            Some(h) => h,
            None => return,
        };
        SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket| socket.close());
        SOCKET_SET.poll_interfaces();
    }

    /// Receives data from the socket, stores it in the given buffer.
    pub fn recv(&self, buf: &mut [u8], timeout_ticks: Option<u64>) -> LinuxResult<usize> {
        if self.is_connecting() {
            bail!(EAGAIN);
        } else if !self.is_connected() {
            bail!(ENOTCONN, "not connected");
        }

        let deadline = timeout_ticks.map(|ticks| current_ticks() + ticks);

        // SAFETY: `self.handle` should be initialized in a connected socket.
        let handle = unsafe { self.handle.get().read().unwrap() };
        self.block_on(|| {
            SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                if socket.recv_queue() > 0 {
                    // data available
                    // TODO: use socket.recv(|buf| {...})
                    let len = socket
                        .recv_slice(buf)
                        .map_err(|_| ax_err!(ENOTCONN, "not connected?"))?;
                    Ok(len)
                } else if !socket.is_active() {
                    // not open
                    bail!(ECONNREFUSED, "connection refused");
                } else if !socket.may_recv() {
                    // connection closed
                    Ok(0)
                } else if deadline.is_some_and(|d| current_ticks() > d) {
                    Err(LinuxError::ETIMEDOUT)
                } else {
                    // no more data
                    Err(LinuxError::EAGAIN)
                }
            })
        })
    }

    /// Transmits data in the given buffer.
    pub fn send(&self, buf: &[u8]) -> LinuxResult<usize> {
        if self.is_connecting() {
            bail!(EAGAIN);
        } else if !self.is_connected() {
            bail!(ENOTCONN);
        }

        // SAFETY: `self.handle` should be initialized in a connected socket.
        let handle = unsafe { self.handle.get().read().unwrap() };
        self.block_on(|| {
            SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                if !socket.is_active() || !socket.may_send() {
                    // closed by remote
                    bail!(ECONNRESET, "connection reset by peer");
                } else if socket.can_send() {
                    // connected, and the tx buffer is not full
                    // TODO: use socket.send(|buf| {...})
                    let len = socket
                        .send_slice(buf)
                        .map_err(|_| ax_err!(ENOTCONN, "not connected?"))?;
                    Ok(len)
                } else {
                    // tx buffer is full
                    Err(LinuxError::EAGAIN)
                }
            })
        })
    }

    /// Whether the socket is readable or writable.
    pub fn poll(&self) -> LinuxResult<PollState> {
        match self.get_state() {
            STATE_CONNECTING => self.poll_connect(),
            STATE_CONNECTED => self.poll_stream(),
            STATE_LISTENING => self.poll_listener(),
            _ => Ok(PollState {
                readable: false,
                writable: false,
            }),
        }
    }

    /// Checks if Nagle's algorithm is enabled for this TCP socket.
    #[inline]
    pub fn nodelay(&self) -> LinuxResult<bool> {
        if let Some(h) = unsafe { self.handle.get().read() } {
            Ok(SOCKET_SET.with_socket::<tcp::Socket, _, _>(h, |socket| socket.nagle_enabled()))
        } else {
            Err(LinuxError::ENOTCONN)
        }
    }

    /// Enables or disables Nagle's algorithm for this TCP socket.
    #[inline]
    pub fn set_nodelay(&self, enabled: bool) -> LinuxResult<()> {
        if let Some(h) = unsafe { self.handle.get().read() } {
            SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(h, |socket| {
                socket.set_nagle_enabled(enabled);
            });
            Ok(())
        } else {
            Err(LinuxError::ENOTCONN)
        }
    }

    /// Returns the maximum capacity of the receive buffer in bytes.
    #[inline]
    pub fn recv_capacity(&self) -> LinuxResult<usize> {
        if let Some(h) = unsafe { self.handle.get().read() } {
            Ok(SOCKET_SET.with_socket::<tcp::Socket, _, _>(h, |socket| socket.recv_capacity()))
        } else {
            Err(LinuxError::ENOTCONN)
        }
    }

    /// Returns the maximum capacity of the send buffer in bytes.
    #[inline]
    pub fn send_capacity(&self) -> LinuxResult<usize> {
        if let Some(h) = unsafe { self.handle.get().read() } {
            Ok(SOCKET_SET.with_socket::<tcp::Socket, _, _>(h, |socket| socket.send_capacity()))
        } else {
            Err(LinuxError::ENOTCONN)
        }
    }
}

/// Private methods
impl TcpSocket {
    #[inline]
    fn get_state(&self) -> u8 {
        self.state.load(Ordering::Acquire)
    }

    #[inline]
    fn set_state(&self, state: u8) {
        self.state.store(state, Ordering::Release);
    }

    /// Update the state of the socket atomically.
    ///
    /// If the current state is `expect`, it first changes the state to
    /// `STATE_BUSY`, then calls the given function. If the function returns
    /// `Ok`, it changes the state to `new`, otherwise it changes the state
    /// back to `expect`.
    ///
    /// It returns `Ok` if the current state is `expect`, otherwise it returns
    /// the current state in `Err`.
    fn update_state<F, T>(&self, expect: u8, new: u8, f: F) -> Result<LinuxResult<T>, u8>
    where
        F: FnOnce() -> LinuxResult<T>,
    {
        match self
            .state
            .compare_exchange(expect, STATE_BUSY, Ordering::Acquire, Ordering::Acquire)
        {
            Ok(_) => {
                let res = f();
                if res.is_ok() {
                    self.set_state(new);
                } else {
                    self.set_state(expect);
                }
                Ok(res)
            }
            Err(old) => Err(old),
        }
    }

    #[inline]
    fn is_connecting(&self) -> bool {
        self.get_state() == STATE_CONNECTING
    }

    #[inline]
    /// Whether the socket is connected.
    pub fn is_connected(&self) -> bool {
        self.get_state() == STATE_CONNECTED
    }

    #[inline]
    /// Whether the socket is closed.
    pub fn is_closed(&self) -> bool {
        self.get_state() == STATE_CLOSED
    }

    #[inline]
    fn is_listening(&self) -> bool {
        self.get_state() == STATE_LISTENING
    }

    fn bound_endpoint(&self) -> LinuxResult<IpListenEndpoint> {
        // SAFETY: no other threads can read or write `self.local_addr`.
        let local_addr = unsafe { self.local_addr.get().read() };
        let port = if local_addr.port != 0 {
            local_addr.port
        } else {
            get_ephemeral_port()?
        };
        assert_ne!(port, 0);
        let addr = if !local_addr.addr.is_unspecified() {
            Some(local_addr.addr)
        } else {
            None
        };
        Ok(IpListenEndpoint { addr, port })
    }

    fn poll_connect(&self) -> LinuxResult<PollState> {
        // SAFETY: `self.handle` should be initialized above.
        let handle = unsafe { self.handle.get().read().unwrap() };
        let writable =
            SOCKET_SET.with_socket::<tcp::Socket, _, _>(handle, |socket| match socket.state() {
                State::SynSent => false, // wait for connection
                State::Established => {
                    self.set_state(STATE_CONNECTED); // connected
                    debug!(
                        "TCP socket {}: connected to {}",
                        handle,
                        socket.remote_endpoint().unwrap(),
                    );
                    true
                }
                _ => {
                    unsafe {
                        self.local_addr.get().write(UNSPECIFIED_ENDPOINT);
                        self.peer_addr.get().write(UNSPECIFIED_ENDPOINT);
                    }
                    self.set_state(STATE_CLOSED); // connection failed
                    true
                }
            });
        Ok(PollState {
            readable: false,
            writable,
        })
    }

    fn poll_stream(&self) -> LinuxResult<PollState> {
        // SAFETY: `self.handle` should be initialized in a connected socket.
        let handle = unsafe { self.handle.get().read().unwrap() };
        SOCKET_SET.with_socket::<tcp::Socket, _, _>(handle, |socket| {
            Ok(PollState {
                readable: !socket.may_recv() || socket.can_recv(),
                writable: !socket.may_send() || socket.can_send(),
            })
        })
    }

    fn poll_listener(&self) -> LinuxResult<PollState> {
        // SAFETY: `self.local_addr` should be initialized in a listening socket.
        let local_addr = unsafe { self.local_addr.get().read() };
        Ok(PollState {
            readable: LISTEN_TABLE.can_accept(local_addr.port)?,
            writable: false,
        })
    }

    /// Block the current thread until the given function completes or fails.
    ///
    /// If the socket is non-blocking, it calls the function once and returns
    /// immediately. Otherwise, it may call the function multiple times if it
    /// returns [`Err(WouldBlock)`](LinuxError::EAGAIN).
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
}

impl Read for TcpSocket {
    fn read(&mut self, buf: &mut [u8]) -> LinuxResult<usize> {
        self.recv(buf, None)
    }
}

impl Write for TcpSocket {
    fn write(&mut self, buf: &[u8]) -> LinuxResult<usize> {
        self.send(buf)
    }

    fn flush(&mut self) -> LinuxResult {
        // TODO(mivik): flush
        Ok(())
    }
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        self.shutdown().ok();
        // Safe because we have mut reference to `self`.
        if let Some(handle) = unsafe { self.handle.get().read() } {
            SOCKET_SET.remove(handle);
        }
    }
}

fn get_ephemeral_port() -> LinuxResult<u16> {
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
    bail!(EADDRINUSE, "no available ports");
}
