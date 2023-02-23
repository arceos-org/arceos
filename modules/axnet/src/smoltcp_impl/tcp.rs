use axerror::{ax_err, ax_err_type, AxError, AxResult};
use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp::{self, ConnectError, ListenError, RecvError, State};
use smoltcp::wire::IpAddress;
use spin::Mutex;

use super::{SocketSet, ETH0, SOCKET_SET};
use crate::SocketAddr;

pub struct TcpSocket {
    handle: SocketHandle,
    local_addr: Option<SocketAddr>,
    peer_addr: Option<SocketAddr>,
}

impl TcpSocket {
    pub fn new() -> Self {
        let socket = SocketSet::new_tcp_socket();
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
        let local_port = gen_local_port();
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
            let state = SOCKET_SET
                .with_socket_mut::<tcp::Socket, _, _>(self.handle, |socket| socket.state());
            // TODO: check host unreachable
            match state {
                State::SynSent => axtask::yield_now(),
                State::Established => {
                    self.local_addr = local_addr;
                    self.peer_addr = peer_addr;
                    return Ok(());
                }
                _ => return ax_err!(ConnectionRefused, "socket connect() failed"),
            }
        }
    }

    pub fn bind(&mut self, addr: SocketAddr) -> AxResult {
        // TODO: check addr is valid
        let mut addr = addr;
        if addr.port == 0 {
            addr.port = gen_local_port();
        }
        self.local_addr = Some(addr);
        Ok(())
    }

    pub fn listen(&mut self) -> AxResult {
        if self.local_addr.is_none() {
            let addr = IpAddress::v4(0, 0, 0, 0);
            let port = gen_local_port();
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
            let (is_active, peer_addr) = SOCKET_SET
                .with_socket_mut::<tcp::Socket, _, _>(self.handle, |socket| {
                    (socket.is_active(), socket.remote_endpoint())
                });

            if is_active {
                debug!(
                    "socket {}: accepted a new connection {}",
                    self.handle,
                    peer_addr.unwrap()
                );

                // return the current socket
                let ret = TcpSocket {
                    handle: self.handle,
                    local_addr: self.local_addr,
                    peer_addr,
                };

                // create a new socket for next connection
                let mut socket = SocketSet::new_tcp_socket();
                socket
                    .listen(self.local_addr.unwrap())
                    .map_err(|_| AxError::BadState)?;
                self.handle = SOCKET_SET.add(socket);

                return Ok(ret);
            }
            axtask::yield_now();
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

fn gen_local_port() -> u16 {
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
