use alloc::vec;
use core::{
    net::{Ipv4Addr, SocketAddr},
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
    task::Context,
};

use axerrno::{LinuxError, LinuxResult, ax_err, bail};
use axio::{
    IoEvents, PollSet, Pollable,
    buf::{Buf, BufMut, BufMutExt},
};
use axsync::Mutex;
use smoltcp::{
    iface::SocketHandle,
    socket::tcp as smol,
    time::Duration,
    wire::{IpEndpoint, IpListenEndpoint},
};

use super::{LISTEN_TABLE, SOCKET_SET};
use crate::{
    RecvFlags, RecvOptions, SERVICE, SendOptions, Shutdown, Socket, SocketAddrEx, SocketOps,
    consts::{TCP_RX_BUF_LEN, TCP_TX_BUF_LEN},
    general::GeneralOptions,
    options::{Configurable, GetSocketOption, SetSocketOption},
    poll_interfaces,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Busy,
    Connecting,
    Connected,
    Listening,
    Closed,
}
impl TryFrom<u8> for State {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        Ok(match value {
            0 => State::Idle,
            1 => State::Busy,
            2 => State::Connecting,
            3 => State::Connected,
            4 => State::Listening,
            5 => State::Closed,
            _ => return Err(()),
        })
    }
}

struct StateLock(AtomicU8);
impl StateLock {
    fn new(state: State) -> Self {
        Self(AtomicU8::new(state as u8))
    }

    fn get(&self) -> State {
        self.0
            .load(Ordering::Acquire)
            .try_into()
            .expect("invalid state")
    }

    fn set(&self, state: State) {
        self.0.store(state as u8, Ordering::Release);
    }

    fn lock(&self, expect: State) -> Result<StateGuard, State> {
        match self.0.compare_exchange(
            expect as u8,
            State::Busy as u8,
            Ordering::Acquire,
            Ordering::Acquire,
        ) {
            Ok(_) => Ok(StateGuard(self, expect as u8)),
            Err(old) => Err(old.try_into().expect("invalid state")),
        }
    }
}

#[must_use]
struct StateGuard<'a>(&'a StateLock, u8);
impl StateGuard<'_> {
    pub fn transit<R>(self, new: State, f: impl FnOnce() -> LinuxResult<R>) -> LinuxResult<R> {
        match f() {
            Ok(result) => {
                self.0.0.store(new as u8, Ordering::Release);
                Ok(result)
            }
            Err(err) => {
                self.0.0.store(self.1, Ordering::Release);
                Err(err)
            }
        }
    }
}

pub(crate) fn new_tcp_socket() -> smol::Socket<'static> {
    smol::Socket::new(
        smol::SocketBuffer::new(vec![0; TCP_RX_BUF_LEN]),
        smol::SocketBuffer::new(vec![0; TCP_TX_BUF_LEN]),
    )
}

/// A TCP socket that provides POSIX-like APIs.
pub struct TcpSocket {
    state: StateLock,
    handle: SocketHandle,

    general: GeneralOptions,
    rx_closed: AtomicBool,
    poll_rx_closed: PollSet,
}

unsafe impl Sync for TcpSocket {}

impl TcpSocket {
    /// Creates a new TCP socket.
    pub fn new() -> Self {
        Self {
            state: StateLock::new(State::Idle),
            handle: SOCKET_SET.add(new_tcp_socket()),

            general: GeneralOptions::new(),
            rx_closed: AtomicBool::new(false),
            poll_rx_closed: PollSet::new(),
        }
    }

    /// Creates a new TCP socket that is already connected.
    fn new_connected(handle: SocketHandle) -> Self {
        Self {
            state: StateLock::new(State::Connected),
            handle,

            general: GeneralOptions::new(),
            rx_closed: AtomicBool::new(false),
            poll_rx_closed: PollSet::new(),
        }
    }
}

/// Private methods
impl TcpSocket {
    fn state(&self) -> State {
        self.state.get()
    }

    #[inline]
    fn is_listening(&self) -> bool {
        self.state() == State::Listening
    }

    fn with_smol_socket<R>(&self, f: impl FnOnce(&mut smol::Socket) -> R) -> R {
        SOCKET_SET.with_socket_mut::<smol::Socket, _, _>(self.handle, f)
    }

    fn bound_endpoint(&self) -> LinuxResult<IpListenEndpoint> {
        let endpoint = self.with_smol_socket(|socket| socket.get_bound_endpoint());
        if endpoint.port == 0 {
            bail!(EINVAL, "not bound");
        }
        Ok(endpoint)
    }

    fn poll_connect(&self) -> IoEvents {
        let mut events = IoEvents::empty();
        let writable = self.with_smol_socket(|socket| match socket.state() {
            smol::State::SynSent => false, // wait for connection
            smol::State::Established => {
                self.state.set(State::Connected); // connected
                debug!(
                    "TCP socket {}: connected to {}",
                    self.handle,
                    socket.remote_endpoint().unwrap(),
                );
                true
            }
            _ => {
                self.state.set(State::Closed); // connection failed
                true
            }
        });
        events.set(IoEvents::OUT, writable);
        events
    }

    fn poll_stream(&self) -> IoEvents {
        let mut events = IoEvents::empty();
        self.with_smol_socket(|socket| {
            events.set(
                IoEvents::IN,
                !self.rx_closed.load(Ordering::Acquire)
                    && (!socket.may_recv() || socket.can_recv()),
            );
            events.set(IoEvents::OUT, !socket.may_send() || socket.can_send());
        });
        events
    }

    fn poll_listener(&self) -> IoEvents {
        let mut events = IoEvents::empty();
        events.set(
            IoEvents::IN,
            LISTEN_TABLE
                .can_accept(self.bound_endpoint().unwrap().port)
                .unwrap(),
        );
        events
    }
}

impl Configurable for TcpSocket {
    fn get_option_inner(&self, option: &mut GetSocketOption) -> LinuxResult<bool> {
        use GetSocketOption as O;

        if self.general.get_option_inner(option)? {
            return Ok(true);
        }

        match option {
            O::NoDelay(no_delay) => {
                **no_delay = self.with_smol_socket(|socket| !socket.nagle_enabled());
            }
            O::KeepAlive(keep_alive) => {
                **keep_alive = self.with_smol_socket(|socket| socket.keep_alive().is_some());
            }
            O::MaxSegment(max_segment) => {
                // TODO(mivik): get actual MSS
                **max_segment = 1460;
            }
            O::SendBuffer(size) => {
                **size = TCP_TX_BUF_LEN;
            }
            O::ReceiveBuffer(size) => {
                **size = TCP_RX_BUF_LEN;
            }
            O::TcpInfo(_) => {
                // TODO(mivik): implement TCP_INFO
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn set_option_inner(&self, option: SetSocketOption) -> LinuxResult<bool> {
        use SetSocketOption as O;

        if self.general.set_option_inner(option)? {
            return Ok(true);
        }

        match option {
            O::NoDelay(no_delay) => {
                self.with_smol_socket(|socket| {
                    socket.set_nagle_enabled(!no_delay);
                });
            }
            O::KeepAlive(keep_alive) => {
                self.with_smol_socket(|socket| {
                    socket.set_keep_alive(keep_alive.then(|| Duration::from_secs(75)));
                });
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl SocketOps for TcpSocket {
    fn bind(&self, local_addr: SocketAddrEx) -> LinuxResult<()> {
        let mut local_addr = local_addr.into_ip()?;
        self.state
            .lock(State::Idle)
            .map_err(|_| ax_err!(EINVAL, "already"))?
            .transit(State::Idle, || {
                // TODO: check addr is available
                if local_addr.port() == 0 {
                    local_addr.set_port(get_ephemeral_port()?);
                }
                if !self.general.reuse_address() {
                    SOCKET_SET.bind_check(local_addr.ip().into(), local_addr.port())?;
                }

                self.with_smol_socket(|socket| {
                    if socket.get_bound_endpoint().port != 0 {
                        return Err(LinuxError::EINVAL);
                    }
                    socket.set_bound_endpoint(IpListenEndpoint {
                        addr: if local_addr.ip().is_unspecified() {
                            None
                        } else {
                            Some(local_addr.ip().into())
                        },
                        port: local_addr.port(),
                    });
                    Ok(())
                })?;
                debug!("TCP socket {}: binding to {}", self.handle, local_addr);
                Ok(())
            })
    }

    fn connect(&self, remote_addr: SocketAddrEx) -> LinuxResult<()> {
        let remote_addr = remote_addr.into_ip()?;
        self.state
            .lock(State::Idle)
            .map_err(|state| {
                if state == State::Connecting {
                    LinuxError::EINPROGRESS
                } else {
                    // TODO(mivik): error code
                    ax_err!(EISCONN, "already connected")
                }
            })?
            .transit(State::Connecting, || {
                // TODO: check remote addr unreachable
                // let (bound_endpoint, remote_endpoint) = self.get_endpoint_pair(remote_addr)?;
                let remote_endpoint = IpEndpoint::from(remote_addr);
                let mut bound_endpoint =
                    self.with_smol_socket(|socket| socket.get_bound_endpoint());
                if bound_endpoint.addr.is_none() {
                    bound_endpoint.addr =
                        Some(SERVICE.lock().get_source_address(&remote_endpoint.addr));
                }
                if bound_endpoint.port == 0 {
                    bound_endpoint.port = get_ephemeral_port()?;
                }
                info!(
                    "TCP connection from {} to {}",
                    bound_endpoint, remote_endpoint
                );

                self.general
                    .set_externally_driven(SERVICE.lock().is_external(&remote_endpoint.addr));

                self.with_smol_socket(|socket| {
                    socket.set_bound_endpoint(bound_endpoint);
                    socket
                        .connect(
                            crate::SERVICE.lock().iface.context(),
                            remote_endpoint,
                            bound_endpoint,
                        )
                        .map_err(|e| match e {
                            smol::ConnectError::InvalidState => {
                                ax_err!(EISCONN, "already conncted")
                            }
                            smol::ConnectError::Unaddressable => {
                                ax_err!(ECONNREFUSED, "unaddressable")
                            }
                        })?;
                    Ok(())
                })
            })?;

        // Here our state must be `CONNECTING`, and only one thread can run here.
        self.general.send_poller(self).poll(|| {
            poll_interfaces();
            let events = self.poll_connect();
            if !events.contains(IoEvents::OUT) {
                Err(LinuxError::EAGAIN)
            } else if self.state() == State::Connected {
                Ok(())
            } else {
                Err(ax_err!(ECONNREFUSED, "connection refused"))
            }
        })
    }

    fn listen(&self) -> LinuxResult<()> {
        if let Ok(guard) = self.state.lock(State::Idle) {
            guard.transit(State::Listening, || {
                let bound_endpoint = self.with_smol_socket(|socket| socket.get_bound_endpoint());
                LISTEN_TABLE.listen(bound_endpoint, &self.general.poll_rx)?;
                debug!("listening on {}", bound_endpoint);
                Ok(())
            })?;
        } else {
            // ignore simultaneous `listen`s.
        }
        Ok(())
    }

    fn accept(&self) -> LinuxResult<Socket> {
        if !self.is_listening() {
            bail!(EINVAL, "not listening");
        }

        let bound_port = self.bound_endpoint()?.port;
        self.general.recv_poller(self).poll(|| {
            poll_interfaces();
            LISTEN_TABLE.accept(bound_port).map(|handle| {
                let socket = TcpSocket::new_connected(handle);
                debug!(
                    "accepted connection from {}, {}",
                    handle,
                    socket.with_smol_socket(|socket| socket.remote_endpoint().unwrap())
                );
                Socket::Tcp(socket)
            })
        })
    }

    fn send(&self, src: &mut impl Buf, _options: SendOptions) -> LinuxResult<usize> {
        // SAFETY: `self.handle` should be initialized in a connected socket.
        self.general.send_poller(self).poll(|| {
            self.with_smol_socket(|socket| {
                if !socket.is_active() {
                    Err(LinuxError::ENOTCONN)
                } else if !socket.can_send() {
                    Err(LinuxError::EAGAIN)
                } else {
                    // connected, and the tx buffer is not full
                    let len = socket
                        .send(|mut buffer| {
                            let len = buffer.put(src);
                            (len, len)
                        })
                        .map_err(|_| ax_err!(ENOTCONN, "not connected?"))?;
                    Ok(len)
                }
            })
        })
    }

    fn recv(&self, dst: &mut impl BufMut, options: RecvOptions<'_>) -> LinuxResult<usize> {
        if self.rx_closed.load(Ordering::Acquire) {
            return Err(LinuxError::ENOTCONN);
        }
        self.general.recv_poller(self).poll(|| {
            poll_interfaces();
            self.with_smol_socket(|socket| {
                if !socket.is_active() {
                    Err(LinuxError::ENOTCONN)
                } else if !socket.may_recv() {
                    Ok(0)
                } else if socket.recv_queue() == 0 {
                    Err(LinuxError::EAGAIN)
                } else {
                    if options.flags.contains(RecvFlags::PEEK) {
                        socket.peek_slice(dst.chunk_mut())
                    } else {
                        socket.recv(|buf| {
                            let len = dst.put(&mut &*buf);
                            (len, len)
                        })
                    }
                    .map_err(|_| ax_err!(ENOTCONN, "not connected?"))
                }
            })
        })
    }

    fn local_addr(&self) -> LinuxResult<SocketAddrEx> {
        self.with_smol_socket(|socket| {
            let endpoint = socket.get_bound_endpoint();
            Ok(SocketAddrEx::Ip(SocketAddr::new(
                endpoint
                    .addr
                    .map_or_else(|| Ipv4Addr::UNSPECIFIED.into(), Into::into),
                endpoint.port,
            )))
        })
    }

    fn peer_addr(&self) -> LinuxResult<SocketAddrEx> {
        self.with_smol_socket(|socket| {
            Ok(SocketAddrEx::Ip(
                socket.remote_endpoint().ok_or(LinuxError::ENOTCONN)?.into(),
            ))
        })
    }

    fn shutdown(&self, how: Shutdown) -> LinuxResult<()> {
        // TODO(mivik): shutdown
        if how.has_read() {
            self.rx_closed.store(true, Ordering::Release);
            self.poll_rx_closed.wake();
        }

        // stream
        if let Ok(guard) = self.state.lock(State::Connected) {
            guard.transit(State::Closed, || {
                if how.has_write() {
                    self.with_smol_socket(|socket| {
                        debug!("TCP socket {}: shutting down", self.handle);
                        socket.close();
                    });
                }
                poll_interfaces();
                Ok(())
            })?;
        }

        // listener
        if let Ok(guard) = self.state.lock(State::Listening) {
            guard.transit(State::Closed, || {
                LISTEN_TABLE.unlisten(self.bound_endpoint()?.port);
                poll_interfaces();
                Ok(())
            })?;
        }

        // ignore for other states
        Ok(())
    }
}

impl Pollable for TcpSocket {
    fn poll(&self) -> IoEvents {
        poll_interfaces();
        let mut events = match self.state() {
            State::Connecting => self.poll_connect(),
            State::Connected | State::Idle | State::Closed => self.poll_stream(),
            State::Listening => self.poll_listener(),
            State::Busy => IoEvents::empty(),
        };
        events.set(IoEvents::RDHUP, self.rx_closed.load(Ordering::Acquire));
        events
    }

    fn register(&self, context: &mut Context<'_>, events: IoEvents) {
        if self.general.externally_driven() {
            context.waker().wake_by_ref();
            return;
        }
        if events.contains(IoEvents::IN) {
            self.general.poll_rx.register(context.waker());
            self.with_smol_socket(|socket| socket.register_recv_waker(&self.general.poll_rx_waker));
            if self.is_listening() {
                LISTEN_TABLE
                    .register_recv_waker(
                        self.bound_endpoint().unwrap().port,
                        &self.general.poll_rx_waker,
                    )
                    .unwrap();
            }
        }
        if events.contains(IoEvents::OUT) {
            self.general.poll_tx.register(context.waker());
            self.with_smol_socket(|socket| socket.register_send_waker(&self.general.poll_tx_waker));
        }
        if events.contains(IoEvents::RDHUP) {
            self.poll_rx_closed.register(context.waker());
        }
    }
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        if let Err(err) = self.shutdown(Shutdown::Both) {
            warn!("TCP socket {}: shutdown failed: {}", self.handle, err);
        }
        SOCKET_SET.remove(self.handle);
        // This is crucial for the close messages to be sent.
        poll_interfaces();
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
