use core::{fmt, str::FromStr};

use lwip_rust::bindings::{
    ip4_addr_t, ip6_addr_t, ip_addr__bindgen_ty_1, ip_addr_t, lwip_ip_addr_type_IPADDR_TYPE_V4,
    lwip_ip_addr_type_IPADDR_TYPE_V6,
};

#[derive(Clone, Copy, Debug)]
pub enum IpAddr {
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
}

#[derive(Clone, Copy, Debug)]
pub struct Ipv4Addr(pub u32);

#[derive(Clone, Copy, Debug)]
pub struct Ipv6Addr {
    pub addr: [u32; 4usize],
    pub zone: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct SocketAddr {
    pub addr: IpAddr,
    pub port: u16,
}

impl FromStr for IpAddr {
    type Err = ();

    fn from_str(s: &str) -> Result<IpAddr, ()> {
        let mut parts = s.split('.');
        let mut addr: u32 = 0;
        for i in 0..4 {
            let part = parts.next().ok_or(())?;
            let part = part.parse::<u8>().map_err(|_| ())?;
            addr |= (part as u32) << (8 * i);
        }
        Ok(IpAddr::Ipv4(Ipv4Addr(addr)))
    }
}

impl Into<ip_addr_t> for IpAddr {
    fn into(self) -> ip_addr_t {
        match self {
            IpAddr::Ipv4(Ipv4Addr(addr)) => ip_addr_t {
                u_addr: ip_addr__bindgen_ty_1 {
                    ip4: ip4_addr_t { addr },
                },
                type_: lwip_ip_addr_type_IPADDR_TYPE_V4 as u8,
            },
            IpAddr::Ipv6(Ipv6Addr { addr, zone }) => ip_addr_t {
                u_addr: ip_addr__bindgen_ty_1 {
                    ip6: ip6_addr_t { addr, zone },
                },
                type_: lwip_ip_addr_type_IPADDR_TYPE_V6 as u8,
            },
        }
    }
}

impl From<ip_addr_t> for IpAddr {
    #[allow(non_upper_case_globals)]
    fn from(addr: ip_addr_t) -> IpAddr {
        match addr.type_ as u32 {
            lwip_ip_addr_type_IPADDR_TYPE_V4 => {
                IpAddr::Ipv4(Ipv4Addr(unsafe { addr.u_addr.ip4.addr }))
            }
            lwip_ip_addr_type_IPADDR_TYPE_V6 => IpAddr::Ipv6(Ipv6Addr {
                addr: unsafe { addr.u_addr.ip6.addr },
                zone: unsafe { addr.u_addr.ip6.zone },
            }),
            _ => panic!("unsupported ip type"),
        }
    }
}

impl fmt::Display for IpAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IpAddr::Ipv4(ipv4_addr) => write!(f, "{ipv4_addr}"),
            IpAddr::Ipv6(ipv6_addr) => write!(f, "{ipv6_addr}"),
        }
    }
}

impl fmt::Display for Ipv4Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.0;
        write!(
            f,
            "{}.{}.{}.{}",
            bytes & 0xff,
            (bytes >> 8) & 0xff,
            (bytes >> 16) & 0xff,
            (bytes >> 24) & 0xff
        )
    }
}

// Reference: https://github.com/smoltcp-rs/smoltcp/blob/9027825c16c9c3fbadb7663e56d64b590fc95d5a/src/wire/ipv6.rs#L247-L306
// Modified to use [u32; 4] instead of [u8; 16]
impl fmt::Display for Ipv6Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The string representation of an IPv6 address should
        // collapse a series of 16 bit sections that evaluate
        // to 0 to "::"
        //
        // See https://tools.ietf.org/html/rfc4291#section-2.2
        // for details.
        enum State {
            Head,
            HeadBody,
            Tail,
            TailBody,
        }
        let mut words = [0u16; 8];
        for i in 0..4 {
            words[i * 2] = ((self.addr[i] & 0xffff) as u16).swap_bytes();
            words[i * 2 + 1] = (((self.addr[i] >> 16) & 0xffff) as u16).swap_bytes();
        }
        let mut state = State::Head;
        for word in words.iter() {
            state = match (*word, &state) {
                // Once a u16 equal to zero write a double colon and
                // skip to the next non-zero u16.
                (0, &State::Head) | (0, &State::HeadBody) => {
                    write!(f, "::")?;
                    State::Tail
                }
                // Continue iterating without writing any characters until
                // we hit a non-zero value.
                (0, &State::Tail) => State::Tail,
                // When the state is Head or Tail write a u16 in hexadecimal
                // without the leading colon if the value is not 0.
                (_, &State::Head) => {
                    write!(f, "{word:x}")?;
                    State::HeadBody
                }
                (_, &State::Tail) => {
                    write!(f, "{word:x}")?;
                    State::TailBody
                }
                // Write the u16 with a leading colon when parsing a value
                // that isn't the first in a section
                (_, &State::HeadBody) | (_, &State::TailBody) => {
                    write!(f, ":{word:x}")?;
                    state
                }
            }
        }
        Ok(())
    }
}

impl FromStr for SocketAddr {
    type Err = ();

    fn from_str(s: &str) -> Result<SocketAddr, ()> {
        let mut parts = s.split(':');
        let addr = parts.next().ok_or(())?.parse::<IpAddr>()?;
        let port = parts.next().ok_or(())?.parse::<u16>().map_err(|_| ())?;
        Ok(SocketAddr { addr, port })
    }
}

impl SocketAddr {
    pub fn new(addr: IpAddr, port: u16) -> SocketAddr {
        SocketAddr { addr, port }
    }
}

impl From<(IpAddr, u16)> for SocketAddr {
    fn from((addr, port): (IpAddr, u16)) -> SocketAddr {
        SocketAddr { addr, port }
    }
}

impl fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.addr, self.port)
    }
}
