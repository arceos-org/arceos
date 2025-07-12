use alloc::vec;
use core::net::SocketAddr;

use axerrno::{LinuxError, LinuxResult, ax_err, bail};
use axio::PollState;
use axsync::Mutex;
use smoltcp::{
    iface::SocketHandle,
    socket::udp as smol,
    storage::PacketMetadata,
    wire::{IpEndpoint, IpListenEndpoint},
};
use spin::RwLock;

use crate::{
    RecvFlags, SOCKET_SET, SendFlags, ShutdownKind, SocketOps,
    consts::{UDP_RX_BUF_LEN, UDP_TX_BUF_LEN, UNSPECIFIED_ENDPOINT_V4},
    general::GeneralOptions,
    options::{Configurable, GetSocketOption, SetSocketOption},
};

pub(crate) fn new_udp_socket() -> smol::Socket<'static> {
    // TODO(mivik): buffer size
    smol::Socket::new(
        smol::PacketBuffer::new(vec![PacketMetadata::EMPTY; 256], vec![0; UDP_RX_BUF_LEN]),
        smol::PacketBuffer::new(vec![PacketMetadata::EMPTY; 256], vec![0; UDP_TX_BUF_LEN]),
    )
}

/// A UDP socket that provides POSIX-like APIs.
pub struct UdpSocket {
    handle: SocketHandle,
    local_addr: RwLock<Option<IpEndpoint>>,
    peer_addr: RwLock<Option<IpEndpoint>>,

    general: GeneralOptions,
}

impl UdpSocket {
    /// Creates a new UDP socket.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let socket = new_udp_socket();
        let handle = SOCKET_SET.add(socket);
        Self {
            handle,
            local_addr: RwLock::new(None),
            peer_addr: RwLock::new(None),

            general: GeneralOptions::new(),
        }
    }

    fn with_smol_socket<R>(&self, f: impl FnOnce(&mut smol::Socket) -> R) -> R {
        SOCKET_SET.with_socket_mut::<smol::Socket, _, _>(self.handle, f)
    }

    fn remote_endpoint(&self) -> LinuxResult<IpEndpoint> {
        match self.peer_addr.try_read() {
            Some(addr) => addr.ok_or(LinuxError::ENOTCONN),
            None => Err(LinuxError::ENOTCONN),
        }
    }
}

impl Configurable for UdpSocket {
    fn get_option_inner(&self, option: &mut GetSocketOption) -> LinuxResult<bool> {
        use GetSocketOption as O;

        if self.general.get_option_inner(option)? {
            return Ok(true);
        }
        match option {
            O::Ttl(ttl) => {
                self.with_smol_socket(|socket| {
                    **ttl = socket.hop_limit().unwrap_or(64);
                });
            }
            O::SendBuffer(size) => {
                **size = UDP_TX_BUF_LEN;
            }
            O::ReceiveBuffer(size) => {
                **size = UDP_RX_BUF_LEN;
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
            O::Ttl(ttl) => {
                self.with_smol_socket(|socket| {
                    socket.set_hop_limit(Some(*ttl));
                });
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl SocketOps for UdpSocket {
    fn bind(&self, mut local_addr: SocketAddr) -> LinuxResult<()> {
        let mut guard = self.local_addr.write();

        if local_addr.port() == 0 {
            local_addr.set_port(get_ephemeral_port()?);
        }
        if guard.is_some() {
            bail!(EINVAL, "already bound");
        }

        let local_endpoint = IpEndpoint::from(local_addr);
        let endpoint = IpListenEndpoint {
            addr: (!local_endpoint.addr.is_unspecified()).then_some(local_endpoint.addr),
            port: local_endpoint.port,
        };

        if !self.general.reuse_address() {
            // Check if the address is already in use
            SOCKET_SET.bind_check(local_endpoint.addr, local_endpoint.port)?;
        }

        self.with_smol_socket(|socket| {
            socket.bind(endpoint).map_err(|e| match e {
                smol::BindError::InvalidState => ax_err!(EINVAL, "already bound"),
                smol::BindError::Unaddressable => ax_err!(ECONNREFUSED, "unaddressable"),
            })
        })?;

        *guard = Some(local_endpoint);
        info!("UDP socket {}: bound on {}", self.handle, endpoint);
        Ok(())
    }

    fn connect(&self, remote_addr: SocketAddr) -> LinuxResult<()> {
        let mut guard = self.peer_addr.write();
        if self.local_addr.read().is_none() {
            self.bind(UNSPECIFIED_ENDPOINT_V4.into())?;
        }

        *guard = Some(remote_addr.into());
        debug!("UDP socket {}: connected to {}", self.handle, remote_addr);
        Ok(())
    }

    fn send(&self, buf: &[u8], to: Option<SocketAddr>, _flags: SendFlags) -> LinuxResult<usize> {
        let remote_addr = match to {
            Some(addr) => addr.into(),
            None => self.remote_endpoint()?,
        };
        if remote_addr.port == 0 || remote_addr.addr.is_unspecified() {
            bail!(EINVAL, "invalid address");
        }

        if self.local_addr.read().is_none() {
            bail!(ENOTCONN);
        }
        self.general.block_on(self.general.send_timeout(), || {
            self.with_smol_socket(|socket| {
                if !socket.is_open() {
                    // not connected
                    bail!(ENOTCONN);
                } else if !socket.can_send() {
                    return Err(LinuxError::EAGAIN);
                }

                socket.send_slice(buf, remote_addr).map_err(|e| match e {
                    smol::SendError::BufferFull => ax_err!(EAGAIN),
                    smol::SendError::Unaddressable => ax_err!(ECONNREFUSED, "unaddressable"),
                })?;
                Ok(buf.len())
            })
        })
    }

    fn recv(
        &self,
        buf: &mut [u8],
        from: Option<&mut SocketAddr>,
        flags: RecvFlags,
    ) -> LinuxResult<usize> {
        if self.local_addr.read().is_none() {
            bail!(ENOTCONN);
        }

        enum ExpectedRemote<'a> {
            Any(&'a mut SocketAddr),
            Expecting(IpEndpoint),
        }
        let mut expected_remote = match from {
            Some(addr) => ExpectedRemote::Any(addr),
            None => ExpectedRemote::Expecting(self.remote_endpoint()?),
        };

        self.general.block_on(self.general.recv_timeout(), || {
            self.with_smol_socket(|socket| {
                if !socket.is_open() {
                    // not bound
                    bail!(ENOTCONN);
                } else if !socket.can_recv() {
                    return Err(LinuxError::EAGAIN);
                }

                let result = if flags.contains(RecvFlags::PEEK) {
                    socket.peek().map(|(data, meta)| (data, *meta))
                } else {
                    socket.recv()
                };
                match result {
                    Ok((src, meta)) => {
                        match &mut expected_remote {
                            ExpectedRemote::Any(remote_addr) => {
                                **remote_addr = meta.endpoint.into();
                            }
                            ExpectedRemote::Expecting(expected) => {
                                if (!expected.addr.is_unspecified()
                                    && expected.addr != meta.endpoint.addr)
                                    || (expected.port != 0 && expected.port != meta.endpoint.port)
                                {
                                    return Err(LinuxError::EAGAIN);
                                }
                            }
                        }

                        let read = src.len().min(buf.len());
                        buf[..read].copy_from_slice(&src[..read]);
                        if read < src.len() {
                            warn!("UDP message truncated: {} -> {} bytes", src.len(), read);
                        }

                        Ok(if flags.contains(RecvFlags::TRUNCATE) {
                            src.len()
                        } else {
                            read
                        })
                    }
                    Err(smol::RecvError::Exhausted) => Err(LinuxError::EAGAIN),
                    Err(smol::RecvError::Truncated) => {
                        unreachable!("UDP socket recv never returns Err(Truncated)")
                    }
                }
            })
        })
    }

    fn local_addr(&self) -> LinuxResult<SocketAddr> {
        match self.local_addr.try_read() {
            Some(addr) => addr.map(Into::into).ok_or(LinuxError::ENOTCONN),
            None => Err(LinuxError::ENOTCONN),
        }
    }

    fn peer_addr(&self) -> LinuxResult<SocketAddr> {
        self.remote_endpoint().map(Into::into)
    }

    fn poll(&self) -> LinuxResult<PollState> {
        if self.local_addr.read().is_none() {
            return Ok(PollState {
                readable: false,
                writable: false,
            });
        }
        self.with_smol_socket(|socket| {
            Ok(PollState {
                readable: socket.can_recv(),
                writable: socket.can_send(),
            })
        })
    }

    fn shutdown(&self, _kind: ShutdownKind) -> LinuxResult<()> {
        // TODO(mivik): shutdown kind
        SOCKET_SET.poll_interfaces();

        self.with_smol_socket(|socket| {
            debug!("UDP socket {}: shutting down", self.handle);
            socket.close();
        });
        Ok(())
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        self.shutdown(ShutdownKind::default()).ok();
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
