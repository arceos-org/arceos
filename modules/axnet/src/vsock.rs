// pub(crate) mod dgram; todo

pub(crate) mod connection_manager;
pub(crate) mod stream;

use core::task::Context;

use axerrno::{AxError, AxResult};
use axio::{Buf, BufMut};
use axpoll::{IoEvents, Pollable};
use enum_dispatch::enum_dispatch;

pub use self::stream::VsockStreamTransport;
use crate::{
    RecvOptions, SendOptions, Shutdown, Socket, SocketAddrEx, SocketOps,
    options::{Configurable, GetSocketOption, SetSocketOption},
};

/// Virtio socket address.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VsockAddr {
    /// Context ID
    pub cid: u32,
    /// Port number
    pub port: u32,
}

impl VsockAddr {
    pub fn to_device_addr(&self) -> AxResult<(u32, u32)> {
        Ok((self.cid, self.port))
    }
}

/// Abstract transport trait for Unix sockets.
#[enum_dispatch]
pub trait VsockTransportOps: Configurable + Pollable + Send + Sync {
    fn bind(&self, local_addr: VsockAddr) -> AxResult;
    fn listen(&self) -> AxResult;
    fn connect(&self, peer_addr: VsockAddr) -> AxResult;
    fn accept(&self) -> AxResult<(VsockTransport, VsockAddr)>;
    fn send(&self, src: &mut impl Buf, options: SendOptions) -> AxResult<usize>;
    fn recv(&self, dst: &mut impl BufMut, options: RecvOptions<'_>) -> AxResult<usize>;
    fn shutdown(&self, _how: Shutdown) -> AxResult;
    fn local_addr(&self) -> AxResult<Option<VsockAddr>>;
    fn peer_addr(&self) -> AxResult<Option<VsockAddr>>;
}

#[enum_dispatch(Configurable, VsockTransportOps)]
pub enum VsockTransport {
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

    fn send(&self, src: &mut impl Buf, options: SendOptions) -> AxResult<usize> {
        self.transport.send(src, options)
    }

    fn recv(&self, dst: &mut impl BufMut, options: RecvOptions<'_>) -> AxResult<usize> {
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
