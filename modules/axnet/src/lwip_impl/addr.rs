use lwip_rust::bindings::{
    ip4_addr_t, ip_addr__bindgen_ty_1, ip_addr_t, lwip_ip_addr_type_IPADDR_TYPE_V4,
};

#[derive(Debug)]
pub enum IpAddr {
    Ipv4(Ipv4Addr),
}

#[derive(Debug)]
pub struct Ipv4Addr(pub u32);

#[derive(Debug)]
pub struct SocketAddr {
    pub addr: IpAddr,
    pub port: u16,
}

impl IpAddr {
    pub fn from_str(s: &str) -> Result<IpAddr, ()> {
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
            _ => unimplemented!(),
        }
    }
}

impl From<(IpAddr, u16)> for SocketAddr {
    fn from((addr, port): (IpAddr, u16)) -> SocketAddr {
        SocketAddr { addr, port }
    }
}
