use alloc::vec;
use core::{
    net::{Ipv4Addr, SocketAddr},
    sync::atomic::{AtomicU8, Ordering},
    task::Poll,
};

use axerrno::{LinuxError, LinuxResult, ax_err, bail};
use axio::{
    PollState,
    buf::{Buf, BufMut},
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
    RecvFlags, SERVICE, SendFlags, Shutdown, Socket, SocketAddrEx, SocketOps,
    consts::{TCP_RX_BUF_LEN, TCP_TX_BUF_LEN},
    general::GeneralOptions,
    options::{Configurable, GetSocketOption, SetSocketOption},
    poll_interfaces,
};

// State transitions:
// CLOSED -(connect)-> BUSY -> CONNECTING -> CONNECTED -(shutdown)-> BUSY ->
// CLOSED       |
//       |-(listen)-> BUSY -> LISTENING -(shutdown)-> BUSY -> CLOSED
//       |
//        -(bind)-> BUSY -> CLOSED
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Closed,
    Busy,
    Connecting,
    Connected,
    Listening,
}
impl TryFrom<u8> for State {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        Ok(match value {
            0 => State::Closed,
            1 => State::Busy,
            2 => State::Connecting,
            3 => State::Connected,
            4 => State::Listening,
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

    fn transit(&self, expect: State, new: State) -> Result<StateGuard, State> {
        match self.0.compare_exchange(
            expect as u8,
            State::Busy as u8,
            Ordering::Acquire,
            Ordering::Acquire,
        ) {
            Ok(_) => Ok(StateGuard(self, new as u8)),
            Err(old) => Err(old.try_into().expect("invalid state")),
        }
    }
}

struct StateGuard<'a>(&'a StateLock, u8);
impl Drop for StateGuard<'_> {
    fn drop(&mut self) {
        self.0.0.store(self.1, Ordering::Release);
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
}

unsafe impl Sync for TcpSocket {}

impl TcpSocket {
    /// Creates a new TCP socket.
    pub fn new() -> Self {
        Self {
            state: StateLock::new(State::Closed),
            handle: SOCKET_SET.add(new_tcp_socket()),

            general: GeneralOptions::new(),
        }
    }

    /// Creates a new TCP socket that is already connected.
    fn new_connected(handle: SocketHandle) -> Self {
        Self {
            state: StateLock::new(State::Connected),
            handle,

            general: GeneralOptions::new(),
        }
    }
}

/// Private methods
impl TcpSocket {
    fn state(&self) -> State {
        self.state.get()
    }

    #[inline]
    fn is_connecting(&self) -> bool {
        self.state() == State::Connecting
    }

    #[inline]
    /// Whether the socket is connected.
    pub fn is_connected(&self) -> bool {
        self.state() == State::Connected
    }

    #[inline]
    /// Whether the socket is closed.
    pub fn is_closed(&self) -> bool {
        self.state() == State::Closed
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

    fn poll_connect(&self) -> LinuxResult<PollState> {
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
        Ok(PollState {
            readable: false,
            writable,
        })
    }

    fn poll_stream(&self) -> LinuxResult<PollState> {
        self.with_smol_socket(|socket| {
            Ok(PollState {
                readable: !socket.may_recv() || socket.can_recv(),
                writable: !socket.may_send() || socket.can_send(),
            })
        })
    }

    fn poll_listener(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: LISTEN_TABLE.can_accept(self.bound_endpoint()?.port)?,
            writable: false,
        })
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
        let _guard = self
            .state
            .transit(State::Closed, State::Closed)
            .map_err(|_| ax_err!(EINVAL, "already bound"))?;

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
    }

    fn connect(&self, remote_addr: SocketAddrEx) -> LinuxResult<()> {
        let remote_addr = remote_addr.into_ip()?;
        let guard = self
            .state
            .transit(State::Closed, State::Connecting)
            .map_err(|state| {
                if state == State::Connecting {
                    LinuxError::EINPROGRESS
                } else {
                    // TODO(mivik): error code
                    ax_err!(EISCONN, "already connected")
                }
            })?;
        // TODO: check remote addr unreachable
        // let (bound_endpoint, remote_endpoint) = self.get_endpoint_pair(remote_addr)?;
        let remote_endpoint = IpEndpoint::from(remote_addr);
        let mut bound_endpoint = self.with_smol_socket(|socket| socket.get_bound_endpoint());
        if bound_endpoint.addr.is_none() {
            bound_endpoint.addr = Some(SERVICE.lock().get_source_address(&remote_endpoint.addr));
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
        })?;

        drop(guard);

        // HACK: yield() to let server to listen
        axtask::yield_now();

        // Here our state must be `CONNECTING`, and only one thread can run here.
        self.general
            .block_on(self.general.send_timeout(), |_context| {
                let PollState { writable, .. } = self.poll_connect()?;
                Poll::Ready(if !writable {
                    Err(LinuxError::EAGAIN)
                } else if self.state() == State::Connected {
                    Ok(())
                } else {
                    Err(ax_err!(ECONNREFUSED, "connection refused"))
                })
            })
    }

    fn listen(&self) -> LinuxResult<()> {
        if let Ok(_guard) = self.state.transit(State::Closed, State::Listening) {
            let bound_endpoint = self.with_smol_socket(|socket| socket.get_bound_endpoint());
            LISTEN_TABLE.listen(bound_endpoint)?;
            debug!("listening on {}", bound_endpoint);
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
        self.general
            .block_on(self.general.recv_timeout(), |_context| {
                Poll::Ready(LISTEN_TABLE.accept(bound_port).map(|handle| {
                    let socket = TcpSocket::new_connected(handle);
                    debug!(
                        "accepted connection from {}, {}",
                        handle,
                        socket.with_smol_socket(|socket| socket.remote_endpoint().unwrap())
                    );
                    socket
                }))
            })
            .map(Socket::Tcp)
    }

    fn send(
        &self,
        src: &mut impl Buf,
        _to: Option<SocketAddrEx>,
        _flags: SendFlags,
    ) -> LinuxResult<usize> {
        if self.is_connecting() {
            return Err(ax_err!(EAGAIN));
        } else if !self.is_connected() {
            return Err(ax_err!(ENOTCONN));
        }

        // SAFETY: `self.handle` should be initialized in a connected socket.
        self.general
            .block_on(self.general.send_timeout(), |context| {
                self.with_smol_socket(|socket| {
                    Poll::Ready(if !socket.is_active() {
                        Err(LinuxError::ENOTCONN)
                    } else if !socket.can_send() {
                        socket.register_send_waker(context.waker());
                        return Poll::Pending;
                    } else {
                        // connected, and the tx buffer is not full
                        let len = socket
                            .send(|mut buffer| {
                                let len = buffer.put(src);
                                (len, len)
                            })
                            .map_err(|_| ax_err!(ENOTCONN, "not connected?"))?;
                        Ok(len)
                    })
                })
            })
    }

    fn recv(
        &self,
        dst: &mut impl BufMut,
        _from: Option<&mut SocketAddrEx>,
        flags: RecvFlags,
    ) -> LinuxResult<usize> {
        if self.is_connecting() {
            bail!(EAGAIN);
        }

        self.general
            .block_on(self.general.recv_timeout(), |context| {
                self.with_smol_socket(|socket| {
                    Poll::Ready(if !socket.is_active() {
                        Err(LinuxError::ENOTCONN)
                    } else if !socket.may_recv() {
                        Ok(0)
                    } else if socket.recv_queue() == 0 {
                        socket.register_recv_waker(context.waker());
                        return Poll::Pending;
                    } else {
                        if flags.contains(RecvFlags::PEEK) {
                            socket.peek_slice(dst.chunk_mut())
                        } else {
                            socket.recv(|buf| {
                                let len = dst.put(&mut &*buf);
                                (len, len)
                            })
                        }
                        .map_err(|_| ax_err!(ENOTCONN, "not connected?"))
                    })
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

    fn poll(&self) -> LinuxResult<PollState> {
        match self.state() {
            State::Connecting => self.poll_connect(),
            State::Connected | State::Closed => self.poll_stream(),
            State::Listening => self.poll_listener(),
            _ => Ok(PollState {
                readable: false,
                writable: false,
            }),
        }
    }

    fn shutdown(&self, _how: Shutdown) -> LinuxResult<()> {
        // TODO(mivik): shutdown

        // stream
        if let Ok(_guard) = self.state.transit(State::Connected, State::Closed) {
            self.with_smol_socket(|socket| {
                debug!("TCP socket {}: shutting down", self.handle);
                socket.close();
            });
            poll_interfaces();
        }

        // listener
        if let Ok(_guard) = self.state.transit(State::Listening, State::Closed) {
            LISTEN_TABLE.unlisten(self.bound_endpoint()?.port);
            poll_interfaces();
        }

        // ignore for other states
        Ok(())
    }
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        if let Err(err) = self.shutdown(Shutdown::Both) {
            warn!("TCP socket {}: shutdown failed: {}", self.handle, err);
        }
        SOCKET_SET.remove(self.handle);
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
