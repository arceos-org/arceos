mod cbindings;
use axdriver::NetDevices;
use axerrno::{ax_err, AxResult};
use lwip_rust::lwip_rust_init;

pub struct IpAddr {}
pub struct SocketAddr {}
pub struct Ipv4Addr {}
pub struct TcpSocket {}

impl TcpSocket {
    pub fn new() -> Self {
        Self {}
    }

    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        ax_err!(Unsupported, "LWIP Unsupported")
    }

    pub fn peer_addr(&self) -> AxResult<SocketAddr> {
        ax_err!(Unsupported, "LWIP Unsupported")
    }

    pub fn connect(&mut self, _addr: SocketAddr) -> AxResult {
        ax_err!(Unsupported, "LWIP Unsupported")
    }

    pub fn bind(&mut self, _addr: SocketAddr) -> AxResult {
        ax_err!(Unsupported, "LWIP Unsupported")
    }

    pub fn listen(&mut self) -> AxResult {
        ax_err!(Unsupported, "LWIP Unsupported")
    }

    pub fn accept(&mut self) -> AxResult<TcpSocket> {
        ax_err!(Unsupported, "LWIP Unsupported")
    }

    pub fn shutdown(&self) -> AxResult {
        ax_err!(Unsupported, "LWIP Unsupported")
    }

    pub fn recv(&self, _buf: &mut [u8]) -> AxResult<usize> {
        ax_err!(Unsupported, "LWIP Unsupported")
    }

    pub fn send(&self, _buf: &[u8]) -> AxResult<usize> {
        ax_err!(Unsupported, "LWIP Unsupported")
    }
}

impl Drop for TcpSocket {
    fn drop(&mut self) {}
}

pub(crate) fn init(_net_devs: NetDevices) {
    lwip_rust_init();
}
