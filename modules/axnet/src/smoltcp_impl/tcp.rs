use axerror::{ax_err, ax_err_type, AxError, AxResult};
use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp::{self, ConnectError, ListenError, RecvError, State};
use smoltcp::wire::IpAddress;
use spin::Mutex;

use super::{SocketSetWrapper, ETH0, SOCKET_SET};
use crate::SocketAddr;

pub struct TcpSocket {
    handle: SocketHandle,
    local_addr: Option<SocketAddr>,
    peer_addr: Option<SocketAddr>,
}

impl TcpSocket {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let socket = SocketSetWrapper::new_tcp_socket();
        let handle = SOCKET_SET.add(socket);
        Self {
            handle,
            local_addr: None,
            peer_addr: None,
        }
    }

    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        self.local_addr.ok_or(AxError::NotConnected)
    }

    pub fn peer_addr(&self) -> AxResult<SocketAddr> {
        self.peer_addr.ok_or(AxError::NotConnected)
    }

    pub fn connect(&mut self, addr: SocketAddr) -> AxResult {
        // TODO: check host unreachable
        let local_port = get_ephemeral_port();
        let iface = &ETH0.iface;
        let (local_addr, peer_addr) =
            SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(self.handle, |socket| {
                socket
                    .connect(iface.lock().context(), addr, local_port)
                    .or_else(|e| match e {
                        ConnectError::InvalidState => {
                            ax_err!(AlreadyExists, "socket connect() failed")
                        }
                        ConnectError::Unaddressable => {
                            ax_err!(InvalidParam, "socket connect() failed")
                        }
                    })?;
                Ok((socket.local_endpoint(), socket.remote_endpoint()))
            })?;

        loop {
            SOCKET_SET.poll_interfaces();
            let (state, may_recv) = SOCKET_SET
                .with_socket::<tcp::Socket, _, _>(self.handle, |socket| {
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

    pub fn bind(&mut self, addr: SocketAddr) -> AxResult {
        // TODO: check addr is valid
        let mut addr = addr;
        if addr.port == 0 {
            addr.port = get_ephemeral_port();
        }
        self.local_addr = Some(addr);
        Ok(())
    }

    pub fn listen(&mut self) -> AxResult {
        if self.local_addr.is_none() {
            let addr = IpAddress::v4(0, 0, 0, 0);
            let port = get_ephemeral_port();
            self.local_addr = Some(SocketAddr::new(addr, port));
        }

        SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(self.handle, |socket| {
            socket
                .listen(self.local_addr.unwrap())
                .or_else(|e| match e {
                    ListenError::InvalidState => ax_err!(AlreadyExists, "socket listen() failed"),
                    ListenError::Unaddressable => ax_err!(InvalidParam, "socket listen() failed"),
                })?;
            debug!(
                "socket {}: listening on {}",
                self.handle,
                self.local_addr.unwrap()
            );
            Ok(())
        })
    }

    pub fn accept(&mut self) -> AxResult<TcpSocket> {
        if self.local_addr.is_none() {
            return ax_err!(InvalidParam, "socket accept() failed: no address bound");
        }

        loop {
            SOCKET_SET.poll_interfaces();
            let (connected, local_addr, peer_addr) =
                SOCKET_SET.with_socket::<tcp::Socket, _, _>(self.handle, |socket| {
                    (
                        !matches!(socket.state(), State::Listen | State::SynReceived),
                        socket.local_endpoint(),
                        socket.remote_endpoint(),
                    )
                });

            if connected {
                debug!(
                    "socket {}: accepted a new connection {}",
                    self.handle,
                    peer_addr.unwrap()
                );

                // create a new socket for next connection
                // TODO: prepare sockets when received SYN
                let mut new_socket = SocketSetWrapper::new_tcp_socket();
                new_socket
                    .listen(local_addr.unwrap())
                    .map_err(|_| AxError::BadState)?;
                let new_handle = SOCKET_SET.add(new_socket);
                let old_hanle = core::mem::replace(&mut self.handle, new_handle);

                // return the old socket
                return Ok(TcpSocket {
                    handle: old_hanle,
                    local_addr,
                    peer_addr,
                });
            } else {
                axtask::yield_now();
            }
        }
    }

    pub fn shutdown(&self) -> AxResult {
        let ret = SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(self.handle, |socket| {
            debug!("socket {}: shutting down", self.handle);
            socket.close();
            Ok(())
        });
        SOCKET_SET.poll_interfaces();
        ret
    }

    pub fn recv(&self, buf: &mut [u8]) -> AxResult<usize> {
        loop {
            SOCKET_SET.poll_interfaces();
            match SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(self.handle, |socket| {
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
                    Err(AxError::ResourceBusy)
                }
            }) {
                Ok(n) => {
                    SOCKET_SET.poll_interfaces();
                    return Ok(n);
                }
                Err(AxError::ResourceBusy) => axtask::yield_now(),
                Err(e) => return Err(e),
            }
        }
    }

    pub fn send(&self, buf: &[u8]) -> AxResult<usize> {
        loop {
            SOCKET_SET.poll_interfaces();
            match SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(self.handle, |socket| {
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
                    Err(AxError::ResourceBusy)
                }
            }) {
                Ok(n) => {
                    SOCKET_SET.poll_interfaces();
                    return Ok(n);
                }
                Err(AxError::ResourceBusy) => axtask::yield_now(),
                Err(e) => return Err(e),
            }
        }
    }
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        self.shutdown().ok();
        SOCKET_SET.remove(self.handle);
    }
}

fn get_ephemeral_port() -> u16 {
    // TODO: check port conflict
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
    port
}
