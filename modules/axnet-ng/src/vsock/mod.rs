// pub(crate) mod dgram; todo

pub(crate) mod connection_manager;
pub(crate) mod stream;

use core::task::Context;

pub use axdriver::prelude::{VsockAddr, VsockConnId};
use axerrno::{AxError, AxResult};
use axio::{IoBuf, IoBufMut, Read, Write};
use axpoll::{IoEvents, Pollable};
use enum_dispatch::enum_dispatch;

pub use self::stream::VsockStreamTransport;
use crate::{
    RecvOptions, SendOptions, Shutdown, Socket, SocketAddrEx, SocketOps,
    options::{Configurable, GetSocketOption, SetSocketOption},
};

/// Abstract transport trait for vsock.
#[enum_dispatch]
pub trait VsockTransportOps: Configurable + Pollable + Send + Sync {
    /// Bind the transport to a local address.
    fn bind(&self, local_addr: VsockAddr) -> AxResult;
    /// Start listening for incoming connections.
    fn listen(&self) -> AxResult;
    /// Connect to a remote peer address.
    fn connect(&self, peer_addr: VsockAddr) -> AxResult;
    /// Accept an incoming connection.
    fn accept(&self) -> AxResult<(VsockTransport, VsockAddr)>;
    /// Send data through the transport.
    fn send(&self, src: impl Read + IoBuf, options: SendOptions) -> AxResult<usize>;
    /// Receive data from the transport.
    fn recv(&self, dst: impl Write, options: RecvOptions<'_>) -> AxResult<usize>;
    /// Shutdown the transport.
    fn shutdown(&self, _how: Shutdown) -> AxResult;
    /// Get the local address, if bound.
    fn local_addr(&self) -> AxResult<Option<VsockAddr>>;
    /// Get the peer address, if connected.
    fn peer_addr(&self) -> AxResult<Option<VsockAddr>>;
}

/// Vsock transport type.
#[enum_dispatch(Configurable, VsockTransportOps)]
pub enum VsockTransport {
    /// Stream-oriented vsock transport.
    Stream(VsockStreamTransport),
    // Dgram(VsockDgramVsockTransport),
}

impl Pollable for VsockTransport {
    fn poll(&self) -> IoEvents {
        match self {
            VsockTransport::Stream(stream) => stream.poll(),
            // VsockTransport::Dgram(dgram) => dgram.poll(),
        }
    }

    fn register(&self, context: &mut core::task::Context<'_>, events: IoEvents) {
        match self {
            VsockTransport::Stream(stream) => stream.register(context, events),
            // VsockTransport::Dgram(dgram) => dgram.register(context, events),
        }
    }
}

/// A network socket using the vsock protocol.
pub struct VsockSocket {
    transport: VsockTransport,
}

impl VsockSocket {
    /// Create a new vsock socket with the given transport.
    pub fn new(transport: impl Into<VsockTransport>) -> Self {
        Self {
            transport: transport.into(),
        }
    }
}

impl Configurable for VsockSocket {
    fn get_option_inner(&self, opt: &mut GetSocketOption) -> AxResult<bool> {
        self.transport.get_option_inner(opt)
    }

    fn set_option_inner(&self, opt: SetSocketOption) -> AxResult<bool> {
        self.transport.set_option_inner(opt)
    }
}

impl SocketOps for VsockSocket {
    fn bind(&self, local_addr: SocketAddrEx) -> AxResult {
        let local_addr = local_addr.into_vsock()?;
        self.transport.bind(local_addr)
    }

    fn connect(&self, remote_addr: SocketAddrEx) -> AxResult {
        let remote_addr = remote_addr.into_vsock()?;
        self.transport.connect(remote_addr)
    }

    fn listen(&self) -> AxResult {
        self.transport.listen()
    }

    fn accept(&self) -> AxResult<Socket> {
        self.transport.accept().map(|(transport, _addr)| {
            let socket = VsockSocket::new(transport);
            Socket::Vsock(socket)
        })
    }

    fn send(&self, src: impl Read + IoBuf, options: SendOptions) -> AxResult<usize> {
        self.transport.send(src, options)
    }

    fn recv(&self, dst: impl Write + IoBufMut, options: RecvOptions<'_>) -> AxResult<usize> {
        self.transport.recv(dst, options)
    }

    fn local_addr(&self) -> AxResult<SocketAddrEx> {
        Ok(SocketAddrEx::Vsock(
            self.transport.local_addr()?.ok_or(AxError::NotFound)?,
        ))
    }

    fn peer_addr(&self) -> AxResult<SocketAddrEx> {
        Ok(SocketAddrEx::Vsock(
            self.transport.peer_addr()?.ok_or(AxError::NotFound)?,
        ))
    }

    fn shutdown(&self, how: Shutdown) -> AxResult {
        self.transport.shutdown(how)
    }
}

impl Pollable for VsockSocket {
    fn poll(&self) -> IoEvents {
        self.transport.poll()
    }

    fn register(&self, context: &mut Context<'_>, events: IoEvents) {
        self.transport.register(context, events);
    }
}
