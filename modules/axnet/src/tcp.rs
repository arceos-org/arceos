use alloc::vec;
use core::{
    cell::UnsafeCell,
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
    consts::{TCP_RX_BUF_LEN, TCP_TX_BUF_LEN, UNSPECIFIED_ENDPOINT_V4},
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
const STATE_CLOSED: u8 = 0;
const STATE_BUSY: u8 = 1;
const STATE_CONNECTING: u8 = 2;
const STATE_CONNECTED: u8 = 3;
const STATE_LISTENING: u8 = 4;

pub(crate) fn new_tcp_socket() -> smol::Socket<'static> {
    smol::Socket::new(
        smol::SocketBuffer::new(vec![0; TCP_RX_BUF_LEN]),
        smol::SocketBuffer::new(vec![0; TCP_TX_BUF_LEN]),
    )
}

/// A TCP socket that provides POSIX-like APIs.
pub struct TcpSocket {
    state: AtomicU8,
    handle: SocketHandle,
    local_addr: UnsafeCell<IpEndpoint>,
    peer_addr: UnsafeCell<IpEndpoint>,

    general: GeneralOptions,
}

unsafe impl Sync for TcpSocket {}

impl TcpSocket {
    /// Creates a new TCP socket.
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(STATE_CLOSED),
            handle: SOCKET_SET.add(new_tcp_socket()),
            local_addr: UnsafeCell::new(UNSPECIFIED_ENDPOINT_V4),
            peer_addr: UnsafeCell::new(UNSPECIFIED_ENDPOINT_V4),

            general: GeneralOptions::new(),
        }
    }

    /// Creates a new TCP socket that is already connected.
    fn new_connected(handle: SocketHandle, local_addr: IpEndpoint, peer_addr: IpEndpoint) -> Self {
        Self {
            state: AtomicU8::new(STATE_CONNECTED),
            handle,
            local_addr: UnsafeCell::new(local_addr),
            peer_addr: UnsafeCell::new(peer_addr),

            general: GeneralOptions::new(),
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

    fn with_smol_socket<R>(&self, f: impl FnOnce(&mut smol::Socket) -> R) -> R {
        SOCKET_SET.with_socket_mut::<smol::Socket, _, _>(self.handle, f)
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
        let writable = self.with_smol_socket(|socket| match socket.state() {
            smol::State::SynSent => false, // wait for connection
            smol::State::Established => {
                self.set_state(STATE_CONNECTED); // connected
                debug!(
                    "TCP socket {}: connected to {}",
                    self.handle,
                    socket.remote_endpoint().unwrap(),
                );
                true
            }
            _ => {
                unsafe {
                    self.local_addr.get().write(UNSPECIFIED_ENDPOINT_V4);
                    self.peer_addr.get().write(UNSPECIFIED_ENDPOINT_V4);
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
        self.with_smol_socket(|socket| {
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
        self.update_state(STATE_CLOSED, STATE_CLOSED, || {
            // TODO: check addr is available
            if local_addr.port() == 0 {
                local_addr.set_port(get_ephemeral_port()?);
            }
            let local_endpoint = IpEndpoint::from(local_addr);
            if !self.general.reuse_address() {
                SOCKET_SET.bind_check(local_endpoint.addr, local_endpoint.port)?;
            }

            // SAFETY: no other threads can read or write `self.local_addr` as we
            // have changed the state to `BUSY`.
            unsafe {
                let old = self.local_addr.get().read();
                if old != UNSPECIFIED_ENDPOINT_V4 {
                    return Err(LinuxError::EINVAL);
                }
                self.local_addr.get().write(local_addr.into());
            }
            let bound_endpoint = self.bound_endpoint()?;
            self.with_smol_socket(|socket| {
                socket.set_bound_endpoint(bound_endpoint);
            });
            debug!("TCP socket {}: binding to {}", self.handle, local_addr);
            Ok(())
        })
        .map_err(|_| ax_err!(EINVAL, "already bound"))?
    }

    fn connect(&self, remote_addr: SocketAddrEx) -> LinuxResult<()> {
        let remote_addr = remote_addr.into_ip()?;
        self.update_state(STATE_CLOSED, STATE_CONNECTING, || {
            // TODO: check remote addr unreachable
            // let (bound_endpoint, remote_endpoint) = self.get_endpoint_pair(remote_addr)?;
            let remote_endpoint = IpEndpoint::from(remote_addr);
            let mut bound_endpoint = self.bound_endpoint()?;
            if bound_endpoint.addr.is_none() {
                bound_endpoint.addr =
                    Some(SERVICE.lock().get_source_address(&remote_endpoint.addr));
            }
            info!(
                "TCP connection from {} to {}",
                bound_endpoint, remote_endpoint
            );

            self.general
                .set_externally_driven(SERVICE.lock().is_external(&remote_endpoint.addr));

            let (local_endpoint, remote_endpoint) = self.with_smol_socket(|socket| {
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
            }
            Ok(())
        })
        .map_err(|state| {
            if state == STATE_CONNECTING {
                LinuxError::EINPROGRESS
            } else {
                // TODO(mivik): error code
                ax_err!(EISCONN, "already connected")
            }
        })??;

        // HACK: yield() to let server to listen
        axtask::yield_now();

        // Here our state must be `CONNECTING`, and only one thread can run here.
        self.general
            .block_on(self.general.send_timeout(), |_context| {
                let PollState { writable, .. } = self.poll_connect()?;
                Poll::Ready(if !writable {
                    Err(LinuxError::EAGAIN)
                } else if self.get_state() == STATE_CONNECTED {
                    Ok(())
                } else {
                    Err(ax_err!(ECONNREFUSED, "connection refused"))
                })
            })
    }

    fn listen(&self) -> LinuxResult<()> {
        self.update_state(STATE_CLOSED, STATE_LISTENING, || {
            let bound_endpoint = self.bound_endpoint()?;
            unsafe {
                (*self.local_addr.get()).port = bound_endpoint.port;
            }
            LISTEN_TABLE.listen(bound_endpoint)?;
            debug!("listening on {}", bound_endpoint);
            Ok(())
        })
        .unwrap_or(Ok(())) // ignore simultaneous `listen`s.
    }

    fn accept(&self) -> LinuxResult<Socket> {
        if !self.is_listening() {
            bail!(EINVAL, "not listening");
        }

        // SAFETY: `self.local_addr` should be initialized after `bind()`.
        let local_port = unsafe { self.local_addr.get().read().port };
        self.general
            .block_on(self.general.recv_timeout(), |_context| {
                Poll::Ready(LISTEN_TABLE.accept(local_port).map(
                    |(handle, (local_addr, peer_addr))| {
                        debug!("accepted connection from {}, {}", handle, peer_addr);
                        TcpSocket::new_connected(handle, local_addr, peer_addr)
                    },
                ))
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
        // 为了通过测例，已经`bind`但未`listen`的socket也可以返回地址
        match self.get_state() {
            STATE_CONNECTED | STATE_LISTENING | STATE_CLOSED => Ok(SocketAddrEx::Ip(
                unsafe { self.local_addr.get().read() }.into(),
            )),
            _ => Err(LinuxError::ENOTCONN),
        }
    }

    fn peer_addr(&self) -> LinuxResult<SocketAddrEx> {
        match self.get_state() {
            STATE_CONNECTED | STATE_LISTENING => Ok(SocketAddrEx::Ip(
                unsafe { self.peer_addr.get().read() }.into(),
            )),
            _ => Err(LinuxError::ENOTCONN),
        }
    }

    fn poll(&self) -> LinuxResult<PollState> {
        match self.get_state() {
            STATE_CONNECTING => self.poll_connect(),
            STATE_CONNECTED | STATE_CLOSED => self.poll_stream(),
            STATE_LISTENING => self.poll_listener(),
            _ => Ok(PollState {
                readable: false,
                writable: false,
            }),
        }
    }

    fn shutdown(&self, _how: Shutdown) -> LinuxResult<()> {
        // TODO(mivik): shutdown

        // stream
        self.update_state(STATE_CONNECTED, STATE_CLOSED, || {
            self.with_smol_socket(|socket| {
                debug!("TCP socket {}: shutting down", self.handle);
                socket.close();
            });
            unsafe { self.local_addr.get().write(UNSPECIFIED_ENDPOINT_V4) }; // clear bound address
            poll_interfaces();
            Ok(())
        })
        .unwrap_or(Ok(()))?;

        // listener
        self.update_state(STATE_LISTENING, STATE_CLOSED, || {
            // SAFETY: `self.local_addr` should be initialized in a listening socket,
            // and no other threads can read or write it.
            let local_port = unsafe { self.local_addr.get().read().port };
            unsafe { self.local_addr.get().write(UNSPECIFIED_ENDPOINT_V4) }; // clear bound address
            LISTEN_TABLE.unlisten(local_port);
            poll_interfaces();
            Ok(())
        })
        .unwrap_or(Ok(()))?;

        // ignore for other states
        Ok(())
    }
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        self.shutdown(Shutdown::Both).ok();
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
