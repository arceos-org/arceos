use axerrno::{ax_err, ax_err_type, AxError, AxResult};
use axsync::Mutex;
use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp::{self, ConnectError, RecvError, State};
use smoltcp::wire::IpAddress;

use super::{SocketSetWrapper, ETH0, LISTEN_TABLE, SOCKET_SET};
use crate::SocketAddr;

pub struct TcpSocket {
    handle: Option<SocketHandle>, // `None` if is listening
    local_addr: Option<SocketAddr>,
    peer_addr: Option<SocketAddr>,
}

impl TcpSocket {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let socket = SocketSetWrapper::new_tcp_socket();
        let handle = Some(SOCKET_SET.add(socket));
        Self {
            handle,
            local_addr: None,
            peer_addr: None,
        }
    }

    const fn is_listening(&self) -> bool {
        self.handle.is_none()
    }

    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        self.local_addr.ok_or(AxError::NotConnected)
    }

    pub fn peer_addr(&self) -> AxResult<SocketAddr> {
        self.peer_addr.ok_or(AxError::NotConnected)
    }

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
                    });
                }
                Err(AxError::Again) => axtask::yield_now(),
                Err(e) => return Err(e),
            }
        }
    }

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
