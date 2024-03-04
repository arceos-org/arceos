use alloc::{collections::VecDeque, vec, vec::Vec};
use smoltcp::{
    iface::SocketSet,
    phy::{Device, DeviceCapabilities, Medium},
    time::Instant,
};

use crate::net_impl::LISTEN_TABLE;

pub(crate) struct LoopbackDev {
    pub(crate) queue: VecDeque<Vec<u8>>,
    medium: Medium,
}

impl LoopbackDev {
    pub fn new(medium: Medium) -> Self {
        Self {
            queue: VecDeque::new(),
            medium,
        }
    }
}

fn snoop_tcp_from_ip(buffer: &[u8], sockets: &mut SocketSet) -> Result<(), smoltcp::wire::Error> {
    use crate::SocketAddr;
    use smoltcp::wire::{IpProtocol, Ipv4Packet, TcpPacket};

    let ipv4_packet = Ipv4Packet::new_checked(buffer)?;

    if ipv4_packet.next_header() == IpProtocol::Tcp {
        let tcp_packet = TcpPacket::new_checked(ipv4_packet.payload())?;
        let src_addr = SocketAddr::new(ipv4_packet.src_addr().into(), tcp_packet.src_port());
        let dst_addr = SocketAddr::new(ipv4_packet.dst_addr().into(), tcp_packet.dst_port());
        let is_first = tcp_packet.syn() && !tcp_packet.ack();
        if is_first {
            // create a socket for the first incoming TCP packet, as the later accept() returns.
            LISTEN_TABLE.incoming_tcp_packet(src_addr, dst_addr, sockets);
        }
    }
    Ok(())
}

pub(crate) struct RxTokenScoop {
    buffer: Vec<u8>,
}

pub(crate) struct TxToken<'a> {
    queue: &'a mut VecDeque<Vec<u8>>,
}

impl smoltcp::phy::RxToken for RxTokenScoop {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        f(&mut self.buffer)
    }

    fn preprocess(&self, sockets: &mut SocketSet<'_>) {
        snoop_tcp_from_ip(&self.buffer, sockets).ok();
    }
}

impl<'a> smoltcp::phy::TxToken for TxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = vec![0; len];
        let result = f(&mut buffer);
        self.queue.push_back(buffer);
        result
    }
}

impl Device for LoopbackDev {
    type RxToken<'a> = RxTokenScoop;
    type TxToken<'a> = TxToken<'a>;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut cap = DeviceCapabilities::default();

        cap.max_transmission_unit = 65535;
        cap.medium = self.medium;

        cap
    }

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.queue.pop_front().map(move |buffer| {
            let rx = Self::RxToken { buffer };
            let tx = Self::TxToken {
                queue: &mut self.queue,
            };
            (rx, tx)
        })
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(TxToken {
            queue: &mut self.queue,
        })
    }
}
