pub(crate) mod dgram;
pub(crate) mod stream;

use alloc::{boxed::Box, sync::Arc};
use core::task::Context;

use async_trait::async_trait;
use axerrno::{LinuxError, LinuxResult};
use axfs_ng::{FS_CONTEXT, OpenOptions};
use axfs_ng_vfs::NodeType;
use axio::{
    IoEvents, Pollable,
    buf::{Buf, BufMut},
};
use axsync::Mutex;
use axtask::future::block_on_interruptible;
use enum_dispatch::enum_dispatch;
use hashbrown::HashMap;
use lazy_static::lazy_static;

pub use self::{dgram::DgramTransport, stream::StreamTransport};
use crate::{
    RecvOptions, SendOptions, Shutdown, Socket, SocketAddrEx, SocketOps,
    options::{Configurable, GetSocketOption, SetSocketOption},
};

#[derive(Default, Clone, Debug)]
pub enum UnixSocketAddr {
    #[default]
    Unnamed,
    Abstract(Arc<[u8]>),
    Path(Arc<str>),
}

/// Abstract transport trait for Unix sockets.
#[async_trait]
#[enum_dispatch]
pub trait TransportOps: Configurable + Pollable + Send + Sync {
    fn bind(&self, slot: &BindSlot, local_addr: &UnixSocketAddr) -> LinuxResult<()>;
    fn connect(&self, slot: &BindSlot, local_addr: &UnixSocketAddr) -> LinuxResult<()>;

    async fn accept(&self) -> LinuxResult<(Transport, UnixSocketAddr)>;

    fn send(&self, src: &mut impl Buf, options: SendOptions) -> LinuxResult<usize>;
    fn recv(&self, dst: &mut impl BufMut, options: RecvOptions<'_>) -> LinuxResult<usize>;

    fn shutdown(&self, _how: Shutdown) -> LinuxResult<()> {
        Ok(())
    }
}

#[enum_dispatch(Configurable, TransportOps)]
pub enum Transport {
    Stream(StreamTransport),
    Dgram(DgramTransport),
}
impl Pollable for Transport {
    fn poll(&self) -> IoEvents {
        match self {
            Transport::Stream(stream) => stream.poll(),
            Transport::Dgram(dgram) => dgram.poll(),
        }
    }

    fn register(&self, context: &mut core::task::Context<'_>, events: IoEvents) {
        match self {
            Transport::Stream(stream) => stream.register(context, events),
            Transport::Dgram(dgram) => dgram.register(context, events),
        }
    }
}

#[derive(Default)]
pub struct BindSlot {
    stream: Mutex<Option<stream::Bind>>,
    dgram: Mutex<Option<dgram::Bind>>,
}

lazy_static! {
    static ref ABSTRACT_BINDS: Mutex<HashMap<Arc<[u8]>, BindSlot>> = Mutex::new(HashMap::new());
}

pub(crate) fn with_slot<R>(
    addr: &UnixSocketAddr,
    f: impl FnOnce(&BindSlot) -> LinuxResult<R>,
) -> LinuxResult<R> {
    match addr {
        UnixSocketAddr::Unnamed => Err(LinuxError::EINVAL),
        UnixSocketAddr::Abstract(name) => {
            let binds = ABSTRACT_BINDS.lock();
            if let Some(slot) = binds.get(name) {
                f(slot)
            } else {
                Err(LinuxError::ENOENT)
            }
        }
        UnixSocketAddr::Path(path) => {
            let loc = FS_CONTEXT.lock().resolve(path.as_ref())?;
            if loc.metadata()?.node_type != NodeType::Socket {
                return Err(LinuxError::ENOTSOCK);
            }
            f(loc
                .user_data()
                .as_ref()
                .ok_or(LinuxError::ECONNREFUSED)?
                .downcast_ref::<BindSlot>()
                .ok_or(LinuxError::ENOTSOCK)?)
        }
    }
}
fn with_slot_or_insert<R>(
    addr: &UnixSocketAddr,
    f: impl FnOnce(&BindSlot) -> LinuxResult<R>,
) -> LinuxResult<R> {
    match addr {
        UnixSocketAddr::Unnamed => Err(LinuxError::EINVAL),
        UnixSocketAddr::Abstract(name) => {
            let mut binds = ABSTRACT_BINDS.lock();
            f(binds.entry(name.clone()).or_default())
        }
        UnixSocketAddr::Path(path) => {
            let loc = OpenOptions::new()
                .write(true)
                .create(true)
                .node_type(NodeType::Socket)
                .open(&*FS_CONTEXT.lock(), path.as_ref())?
                .into_location();
            if loc.metadata()?.node_type != NodeType::Socket {
                return Err(LinuxError::ENOTSOCK);
            }
            let mut user_data = loc.user_data();
            let data = user_data.get_or_insert_with(|| Box::new(BindSlot::default()));
            f(data
                .downcast_ref::<BindSlot>()
                .ok_or(LinuxError::ENOTSOCK)?)
        }
    }
}

pub struct UnixSocket {
    transport: Transport,
    local_addr: Mutex<UnixSocketAddr>,
    remote_addr: Mutex<UnixSocketAddr>,
}
impl UnixSocket {
    pub fn new(transport: impl Into<Transport>) -> Self {
        Self {
            transport: transport.into(),
            local_addr: Mutex::new(UnixSocketAddr::Unnamed),
            remote_addr: Mutex::new(UnixSocketAddr::Unnamed),
        }
    }
}
impl Configurable for UnixSocket {
    fn get_option_inner(&self, opt: &mut GetSocketOption) -> LinuxResult<bool> {
        self.transport.get_option_inner(opt)
    }

    fn set_option_inner(&self, opt: SetSocketOption) -> LinuxResult<bool> {
        self.transport.set_option_inner(opt)
    }
}
impl SocketOps for UnixSocket {
    fn bind(&self, local_addr: SocketAddrEx) -> LinuxResult<()> {
        let local_addr = local_addr.into_unix()?;
        let mut guard = self.local_addr.lock();
        if matches!(&*guard, UnixSocketAddr::Unnamed) {
            with_slot_or_insert(&local_addr, |slot| self.transport.bind(slot, &local_addr))?;
            *guard = local_addr;
        } else {
            return Err(LinuxError::EINVAL);
        }
        Ok(())
    }

    fn connect(&self, remote_addr: SocketAddrEx) -> LinuxResult<()> {
        let remote_addr = remote_addr.into_unix()?;
        let local_addr = self.local_addr.lock().clone();
        let mut guard = self.remote_addr.lock();
        if matches!(&*guard, UnixSocketAddr::Unnamed) {
            with_slot(&remote_addr, |slot| {
                self.transport.connect(slot, &local_addr)
            })?;
            *guard = remote_addr;
        } else {
            return Err(LinuxError::EINVAL);
        }
        Ok(())
    }

    fn listen(&self) -> LinuxResult<()> {
        Ok(())
    }

    fn accept(&self) -> LinuxResult<Socket> {
        let (transport, peer_addr) = block_on_interruptible(self.transport.accept())?;
        Ok(Socket::Unix(Self {
            transport,
            local_addr: Mutex::new(self.local_addr.lock().clone()),
            remote_addr: Mutex::new(peer_addr),
        }))
    }

    fn send(&self, src: &mut impl Buf, options: SendOptions) -> LinuxResult<usize> {
        self.transport.send(src, options)
    }

    fn recv(&self, dst: &mut impl BufMut, options: RecvOptions<'_>) -> LinuxResult<usize> {
        self.transport.recv(dst, options)
    }

    fn local_addr(&self) -> LinuxResult<SocketAddrEx> {
        Ok(SocketAddrEx::Unix(self.local_addr.lock().clone()))
    }

    fn peer_addr(&self) -> LinuxResult<SocketAddrEx> {
        Ok(SocketAddrEx::Unix(self.remote_addr.lock().clone()))
    }

    fn shutdown(&self, how: Shutdown) -> LinuxResult<()> {
        self.transport.shutdown(how)
    }
}

impl Pollable for UnixSocket {
    fn poll(&self) -> IoEvents {
        self.transport.poll()
    }

    fn register(&self, context: &mut Context<'_>, events: IoEvents) {
        self.transport.register(context, events);
    }
}
