use core::iter;
use core::option;

extern crate alloc;
use alloc::slice;
use alloc::vec;
use alloc::vec::Vec;

use axerrno;
use axerrno::ax_err_type;
use axnet::{IpAddr, SocketAddr};

use crate::io;

use axnet::resolve_socket_addr;

/// A trait for objects which can be converted or resolved to one or more [`SocketAddr`] values.
pub trait ToSocketAddrs {
    /// Returned iterator over socket addresses which this type may correspond to.
    type Iter: Iterator<Item = SocketAddr>;

    /// Converts this object to an iterator of resolved [`SocketAddr`]s.
    fn to_socket_addrs(&self) -> io::Result<Self::Iter>;
}

impl ToSocketAddrs for SocketAddr {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        Ok(Some(*self).into_iter())
    }
}

impl ToSocketAddrs for (IpAddr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddr::new(ip, port).to_socket_addrs()
    }
}

impl<'a> ToSocketAddrs for &'a [SocketAddr] {
    type Iter = iter::Cloned<slice::Iter<'a, SocketAddr>>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(self.iter().cloned())
    }
}

impl<T: ToSocketAddrs + ?Sized> ToSocketAddrs for &T {
    type Iter = T::Iter;
    fn to_socket_addrs(&self) -> io::Result<T::Iter> {
        (**self).to_socket_addrs()
    }
}

impl ToSocketAddrs for (&str, u16) {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        let (host, port) = *self;

        // try to parse the host as a regular IP address first
        if let Ok(addr) = host.parse::<IpAddr>() {
            let addr = SocketAddr::new(addr, port);
            return Ok(vec![addr].into_iter());
        }

        Ok(resolve_socket_addr(host)?
            .iter()
            .map(|a| SocketAddr::new(*a, port))
            .collect::<Vec<_>>()
            .into_iter())
    }
}

impl ToSocketAddrs for str {
    type Iter = vec::IntoIter<SocketAddr>;

    // split the string by ':' and convert the second part to u16
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        // try to parse as a regular SocketAddr first
        if let Ok(addr) = self.parse() {
            return Ok(vec![addr].into_iter());
        }

        let (host, port_str) = self
            .rsplit_once(':')
            .ok_or_else(|| ax_err_type!(InvalidInput, "invalid socket address"))?;
        let port: u16 = port_str
            .parse()
            .map_err(|_| ax_err_type!(InvalidInput, "invalid port value"))?;
        Ok(resolve_socket_addr(host)?
            .iter()
            .map(|a| SocketAddr::new(*a, port))
            .collect::<Vec<_>>()
            .into_iter())
    }
}
