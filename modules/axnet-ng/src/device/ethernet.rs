use alloc::{string::String, vec};
use core::task::Waker;

use axdriver::prelude::*;
use axtask::future::register_irq_waker;
use hashbrown::HashMap;
use smoltcp::{
    storage::{PacketBuffer, PacketMetadata},
    time::{Duration, Instant},
    wire::{
        ArpOperation, ArpPacket, ArpRepr, EthernetAddress, EthernetFrame, EthernetProtocol,
        EthernetRepr, IpAddress, Ipv4Cidr,
    },
};

use crate::{
    consts::{ETHERNET_MAX_PENDING_PACKETS, STANDARD_MTU},
    device::Device,
};

const EMPTY_MAC: EthernetAddress = EthernetAddress([0; 6]);

struct Neighbor {
    hardware_address: EthernetAddress,
    expires_at: Instant,
}

pub struct EthernetDevice {
    name: String,
    inner: AxNetDevice,
    neighbors: HashMap<IpAddress, Option<Neighbor>>,
    ip: Ipv4Cidr,

    pending_packets: PacketBuffer<'static, IpAddress>,
}
impl EthernetDevice {
    const NEIGHBOR_TTL: Duration = Duration::from_secs(60);

    pub fn new(name: String, inner: AxNetDevice, ip: Ipv4Cidr) -> Self {
        let pending_packets = PacketBuffer::new(
            vec![PacketMetadata::EMPTY; ETHERNET_MAX_PENDING_PACKETS],
            vec![
                0u8;
                (STANDARD_MTU + EthernetFrame::<&[u8]>::header_len())
                    * ETHERNET_MAX_PENDING_PACKETS
            ],
        );
        Self {
            name,
            inner,
            neighbors: HashMap::new(),
            ip,

            pending_packets,
        }
    }

    #[inline]
    fn hardware_address(&self) -> EthernetAddress {
        EthernetAddress(self.inner.mac_address().0)
    }

    fn send_to<F>(
        inner: &mut AxNetDevice,
        dst: EthernetAddress,
        size: usize,
        f: F,
        proto: EthernetProtocol,
    ) where
        F: FnOnce(&mut [u8]),
    {
        if let Err(err) = inner.recycle_tx_buffers() {
            warn!("recycle_tx_buffers failed: {:?}", err);
            return;
        }

        let repr = EthernetRepr {
            src_addr: EthernetAddress(inner.mac_address().0),
            dst_addr: dst,
            ethertype: proto,
        };

        let mut tx_buf = match inner.alloc_tx_buffer(repr.buffer_len() + size) {
            Ok(buf) => buf,
            Err(err) => {
                warn!("alloc_tx_buffer failed: {:?}", err);
                return;
            }
        };
        let mut frame = EthernetFrame::new_unchecked(tx_buf.packet_mut());
        repr.emit(&mut frame);
        f(frame.payload_mut());
        trace!(
            "SEND {} bytes: {:02X?}",
            tx_buf.packet_len(),
            tx_buf.packet()
        );
        if let Err(err) = inner.transmit(tx_buf) {
            warn!("transmit failed: {:?}", err);
        }
    }

    fn handle_frame(
        &mut self,
        frame: &[u8],
        buffer: &mut PacketBuffer<()>,
        timestamp: Instant,
    ) -> bool {
        let frame = EthernetFrame::new_unchecked(frame);
        let Ok(repr) = EthernetRepr::parse(&frame) else {
            warn!("Dropping malformed Ethernet frame");
            return false;
        };

        if !repr.dst_addr.is_broadcast()
            && repr.dst_addr != EMPTY_MAC
            && repr.dst_addr != self.hardware_address()
        {
            return false;
        }

        match repr.ethertype {
            EthernetProtocol::Ipv4 => {
                buffer
                    .enqueue(frame.payload().len(), ())
                    .unwrap()
                    .copy_from_slice(frame.payload());
                return true;
            }
            EthernetProtocol::Arp => self.process_arp(frame.payload(), timestamp),
            _ => {}
        }

        false
    }

    fn request_arp(&mut self, target_ip: IpAddress) {
        let IpAddress::Ipv4(target_ipv4) = target_ip else {
            warn!("IPv6 address ARP is not supported: {}", target_ip);
            return;
        };
        debug!("Requesting ARP for {}", target_ipv4);

        let arp_repr = ArpRepr::EthernetIpv4 {
            operation: ArpOperation::Request,
            source_hardware_addr: self.hardware_address(),
            source_protocol_addr: self.ip.address(),
            target_hardware_addr: EthernetAddress::BROADCAST,
            target_protocol_addr: target_ipv4,
        };

        Self::send_to(
            &mut self.inner,
            EthernetAddress::BROADCAST,
            arp_repr.buffer_len(),
            |buf| arp_repr.emit(&mut ArpPacket::new_unchecked(buf)),
            EthernetProtocol::Arp,
        );

        self.neighbors.insert(target_ip, None);
    }

    fn process_arp(&mut self, payload: &[u8], now: Instant) {
        let Ok(repr) = ArpPacket::new_checked(payload).and_then(|packet| ArpRepr::parse(&packet))
        else {
            warn!("Dropping malformed ARP packet");
            return;
        };

        if let ArpRepr::EthernetIpv4 {
            operation,
            source_hardware_addr,
            source_protocol_addr,
            target_hardware_addr,
            target_protocol_addr,
        } = repr
        {
            let is_unicast_mac =
                target_hardware_addr != EMPTY_MAC && !target_hardware_addr.is_broadcast();
            if is_unicast_mac && self.hardware_address() != target_hardware_addr {
                // Only process packet that are for us
                return;
            }

            if let ArpOperation::Unknown(_) = operation {
                return;
            }

            if !source_hardware_addr.is_unicast()
                || source_protocol_addr.is_broadcast()
                || source_protocol_addr.is_multicast()
                || source_protocol_addr.is_unspecified()
            {
                return;
            }
            if self.ip.address() != target_protocol_addr {
                return;
            }

            debug!("ARP: {} -> {}", source_protocol_addr, source_hardware_addr);
            self.neighbors.insert(
                IpAddress::Ipv4(source_protocol_addr),
                Some(Neighbor {
                    hardware_address: source_hardware_addr,
                    expires_at: now + Self::NEIGHBOR_TTL,
                }),
            );

            if let ArpOperation::Request = operation {
                let response = ArpRepr::EthernetIpv4 {
                    operation: ArpOperation::Reply,
                    source_hardware_addr: self.hardware_address(),
                    source_protocol_addr: self.ip.address(),
                    target_hardware_addr: source_hardware_addr,
                    target_protocol_addr: source_protocol_addr,
                };

                Self::send_to(
                    &mut self.inner,
                    source_hardware_addr,
                    response.buffer_len(),
                    |buf| response.emit(&mut ArpPacket::new_unchecked(buf)),
                    EthernetProtocol::Arp,
                );
            }

            if self
                .pending_packets
                .peek()
                .is_ok_and(|it| it.0 == &IpAddress::Ipv4(source_protocol_addr))
            {
                while let Ok((&next_hop, buf)) = self.pending_packets.peek() {
                    // TODO: optimize logic such that one long-pending ARP
                    // request does not block all other packets

                    let Some(Some(neighbor)) = self.neighbors.get(&next_hop) else {
                        break;
                    };
                    if neighbor.expires_at <= now {
                        // Neighbor is expired, we need to request ARP again
                        self.request_arp(next_hop);
                        break;
                    }

                    Self::send_to(
                        &mut self.inner,
                        neighbor.hardware_address,
                        buf.len(),
                        |b| b.copy_from_slice(buf),
                        EthernetProtocol::Ipv4,
                    );
                    let _ = self.pending_packets.dequeue();
                }
            }
        }
    }
}

impl Device for EthernetDevice {
    fn name(&self) -> &str {
        &self.name
    }

    fn recv(&mut self, buffer: &mut PacketBuffer<()>, timestamp: Instant) -> bool {
        loop {
            let rx_buf = match self.inner.receive() {
                Ok(buf) => buf,
                Err(err) => {
                    if !matches!(err, DevError::Again) {
                        warn!("receive failed: {:?}", err);
                    }
                    return false;
                }
            };
            trace!(
                "RECV {} bytes: {:02X?}",
                rx_buf.packet_len(),
                rx_buf.packet()
            );

            let result = self.handle_frame(rx_buf.packet(), buffer, timestamp);
            self.inner.recycle_rx_buffer(rx_buf).unwrap();
            if result {
                return true;
            }
        }
    }

    fn send(&mut self, next_hop: IpAddress, packet: &[u8], timestamp: Instant) -> bool {
        if next_hop.is_broadcast() || self.ip.broadcast().map(IpAddress::Ipv4) == Some(next_hop) {
            Self::send_to(
                &mut self.inner,
                EthernetAddress::BROADCAST,
                packet.len(),
                |buf| buf.copy_from_slice(packet),
                EthernetProtocol::Ipv4,
            );
            return false;
        }

        let need_request = match self.neighbors.get(&next_hop) {
            Some(Some(neighbor)) => {
                if neighbor.expires_at > timestamp {
                    Self::send_to(
                        &mut self.inner,
                        neighbor.hardware_address,
                        packet.len(),
                        |buf| buf.copy_from_slice(packet),
                        EthernetProtocol::Ipv4,
                    );
                    return false;
                } else {
                    true
                }
            }
            // Request already sent
            Some(None) => false,
            None => true,
        };
        // Only send ARP request if we haven't already requested it
        if need_request {
            self.request_arp(next_hop);
        }
        if self.pending_packets.is_full() {
            warn!("Pending packets buffer is full, dropping packet");
            return false;
        }
        let Ok(dst_buffer) = self.pending_packets.enqueue(packet.len(), next_hop) else {
            warn!("Failed to enqueue packet in pending packets buffer");
            return false;
        };
        dst_buffer.copy_from_slice(packet);
        false
    }

    fn register_waker(&self, waker: &Waker) {
        if let Some(irq) = self.inner.irq_num() {
            register_irq_waker(irq, waker);
        }
    }
}
