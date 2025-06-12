use core::net::{IpAddr, SocketAddr};
use smoltcp::wire::{IpAddress, IpEndpoint,};

pub  fn from_core_sockaddr(addr: SocketAddr) -> IpEndpoint {
    IpEndpoint {
        addr: IpAddress::from(addr.ip()),
        port: addr.port(),
    }
}

pub  fn into_core_sockaddr(addr: IpEndpoint) -> SocketAddr {
    SocketAddr::new(IpAddr::from(addr.addr), addr.port)
}

pub const UNSPECIFIED_IP: IpAddress = IpAddress::v4(0, 0, 0, 0);
pub const UNSPECIFIED_ENDPOINT: IpEndpoint = IpEndpoint::new(UNSPECIFIED_IP, 0);
