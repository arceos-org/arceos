extern crate alloc;

use crate::io;
use alloc::string::String;
use core::{iter, option, slice};

pub use core::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

/// A trait for objects which can be converted or resolved to one or more
/// [`SocketAddr`] values.
///
/// This trait is used for generic address resolution when constructing network
/// objects. By default it is implemented for the following types:
///
///  * [`SocketAddr`]: [`to_socket_addrs`] is the identity function.
///
///  * [`SocketAddrV4`], <code>([IpAddr], [u16])</code>,
///    <code>([Ipv4Addr], [u16])</code>:
///    [`to_socket_addrs`] constructs a [`SocketAddr`] trivially.
///
///  * <code>(&[str], [u16])</code>: <code>&[str]</code> should be either a string representation
///    of an [`IpAddr`] address as expected by [`FromStr`] implementation or a host
///    name. [`u16`] is the port number.
///
///  * <code>&[str]</code>: the string should be either a string representation of a
///    [`SocketAddr`] as expected by its [`FromStr`] implementation or a string like
///    `<host_name>:<port>` pair where `<port>` is a [`u16`] value.
///
/// [`FromStr`]: core::str::FromStr
/// [`to_socket_addrs`]: ToSocketAddrs::to_socket_addrs
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

impl ToSocketAddrs for SocketAddrV4 {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        SocketAddr::V4(*self).to_socket_addrs()
    }
}

impl ToSocketAddrs for (IpAddr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddr::new(ip, port).to_socket_addrs()
    }
}

impl ToSocketAddrs for (Ipv4Addr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV4::new(ip, port).to_socket_addrs()
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

#[cfg(not(feature = "dns"))]
#[doc(cfg(feature = "net"))]
mod no_dns {
    use super::*;

    impl ToSocketAddrs for (&str, u16) {
        type Iter = option::IntoIter<SocketAddr>;
        fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
            let (host, port) = *self;
            Ok(host
                .parse::<Ipv4Addr>()
                .ok()
                .map(|addr| {
                    let addr = SocketAddrV4::new(addr, port);
                    SocketAddr::V4(addr)
                })
                .into_iter())
        }
    }

    impl ToSocketAddrs for str {
        type Iter = option::IntoIter<SocketAddr>;
        fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
            // parse as a regular SocketAddr first
            Ok(self.parse().ok().into_iter())
        }
    }

    impl ToSocketAddrs for (String, u16) {
        type Iter = option::IntoIter<SocketAddr>;
        fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
            (&*self.0, self.1).to_socket_addrs()
        }
    }

    impl ToSocketAddrs for String {
        type Iter = option::IntoIter<SocketAddr>;
        fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
            (**self).to_socket_addrs()
        }
    }
}

#[cfg(feature = "dns")]
#[doc(cfg(feature = "net"))]
mod dns {
    use super::*;
    use alloc::{vec, vec::Vec};

    impl ToSocketAddrs for (&str, u16) {
        type Iter = vec::IntoIter<SocketAddr>;
        fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
            let (host, port) = *self;

            // try to parse the host as a regular IP address first
            if let Ok(addr) = host.parse::<Ipv4Addr>() {
                let addr = SocketAddrV4::new(addr, port);
                return Ok(vec![SocketAddr::V4(addr)].into_iter());
            }

            Ok(arceos_api::net::ax_dns_query(host)?
                .into_iter()
                .map(|ip| SocketAddr::new(ip, port))
                .collect::<Vec<_>>()
                .into_iter())
        }
    }

    impl ToSocketAddrs for str {
        type Iter = vec::IntoIter<SocketAddr>;

        fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
            // try to parse as a regular SocketAddr first
            if let Ok(addr) = self.parse() {
                return Ok(vec![addr].into_iter());
            }

            // split the string by ':' and convert the second part to u16
            let (host, port_str) = self
                .rsplit_once(':')
                .ok_or_else(|| axerrno::ax_err_type!(InvalidInput, "invalid socket address"))?;
            let port: u16 = port_str
                .parse()
                .map_err(|_| axerrno::ax_err_type!(InvalidInput, "invalid port value"))?;

            Ok(arceos_api::net::ax_dns_query(host)?
                .into_iter()
                .map(|ip| SocketAddr::new(ip, port))
                .collect::<Vec<_>>()
                .into_iter())
        }
    }

    impl ToSocketAddrs for (String, u16) {
        type Iter = vec::IntoIter<SocketAddr>;
        fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
            (&*self.0, self.1).to_socket_addrs()
        }
    }

    impl ToSocketAddrs for String {
        type Iter = vec::IntoIter<SocketAddr>;
        fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
            (**self).to_socket_addrs()
        }
    }
}
