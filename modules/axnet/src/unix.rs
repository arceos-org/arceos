use alloc::sync::Arc;

use axerrno::{LinuxError, LinuxResult};
use axsync::spin::SpinNoIrq;

use crate::{Socket, SocketAddrEx, SocketOps};

#[derive(Clone, Debug)]
pub enum UnixSocketAddr {
    Unnamed,
    Abstract(Arc<[u8]>),
    Path(Arc<str>),
}

enum Transport {
    Tcp {},
    Udp {},
}

pub struct UnixSocket {
    transport: Transport,
    local_addr: SpinNoIrq<Option<SocketAddrEx>>,
}
