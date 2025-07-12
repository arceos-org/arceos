use alloc::vec;
use core::{cell::RefCell, ops::DerefMut};

use axdriver::AxNetDevice;
use axdriver_net::{DevError, NetBufPtr, NetDriverOps};
use axerrno::{LinuxError, LinuxResult};
use axhal::time::{NANOS_PER_MICROS, monotonic_time_nanos};
use axsync::Mutex;
use smoltcp::{
    iface::{Interface, SocketHandle, SocketSet},
    phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken},
    socket::{AnySocket, Socket},
    time::Instant,
    wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr},
};

use crate::{LISTEN_TABLE, LOOPBACK, LOOPBACK_DEV, SOCKET_SET, consts::RANDOM_SEED};

pub(crate) struct SocketSetWrapper<'a>(Mutex<SocketSet<'a>>);

struct DeviceWrapper {
    inner: RefCell<AxNetDevice>, /* use `RefCell` is enough since it's wrapped in `Mutex` in
                                  * `InterfaceWrapper`. */
}

pub(crate) struct InterfaceWrapper {
    name: &'static str,
    ether_addr: EthernetAddress,
    dev: Mutex<DeviceWrapper>,
    pub iface: Mutex<Interface>,
}

impl<'a> SocketSetWrapper<'a> {
    pub fn new() -> Self {
        Self(Mutex::new(SocketSet::new(vec![])))
    }

    pub fn add<T: AnySocket<'a>>(&self, socket: T) -> SocketHandle {
        let handle = self.0.lock().add(socket);
        debug!("socket {}: created", handle);
        handle
    }

    pub fn with_socket<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let set = self.0.lock();
        let socket = set.get(handle);
        f(socket)
    }

    pub fn with_socket_mut<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut set = self.0.lock();
        let socket = set.get_mut(handle);
        f(socket)
    }

    pub fn bind_check(&self, addr: IpAddress, _port: u16) -> LinuxResult {
        // TODO(mivik): optimize
        let mut sockets = self.0.lock();
        for (_, socket) in sockets.iter_mut() {
            match socket {
                Socket::Tcp(s) => {
                    let local_addr = s.get_bound_endpoint();
                    if local_addr.addr == Some(addr) {
                        return Err(LinuxError::EADDRINUSE);
                    }
                }
                Socket::Udp(s) => {
                    if s.endpoint().addr == Some(addr) {
                        return Err(LinuxError::EADDRINUSE);
                    }
                }
                _ => continue,
            };
        }
        Ok(())
    }

    pub fn poll_interfaces(&self) {
        // TODO(mivik): poll delay
        LOOPBACK.lock().poll(
            Instant::from_micros_const((monotonic_time_nanos() / NANOS_PER_MICROS) as i64),
            LOOPBACK_DEV.lock().deref_mut(),
            &mut self.0.lock(),
        );
    }

    pub fn remove(&self, handle: SocketHandle) {
        self.0.lock().remove(handle);
        debug!("socket {}: destroyed", handle);
    }
}

#[allow(unused)]
impl InterfaceWrapper {
    pub fn new(name: &'static str, dev: AxNetDevice, ether_addr: EthernetAddress) -> Self {
        let mut config = smoltcp::iface::Config::new(HardwareAddress::Ethernet(ether_addr));
        // TODO(mivik): random seed
        config.random_seed = RANDOM_SEED;

        let mut dev = DeviceWrapper::new(dev);
        let iface = Mutex::new(Interface::new(config, &mut dev, Self::current_time()));
        Self {
            name,
            ether_addr,
            dev: Mutex::new(dev),
            iface,
        }
    }

    fn current_time() -> Instant {
        Instant::from_micros_const((monotonic_time_nanos() / NANOS_PER_MICROS) as i64)
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn ethernet_address(&self) -> EthernetAddress {
        self.ether_addr
    }

    pub fn setup_ip_addr(&self, ip: IpAddress, prefix_len: u8) {
        let mut iface = self.iface.lock();
        iface.update_ip_addrs(|ip_addrs| {
            ip_addrs.push(IpCidr::new(ip, prefix_len)).unwrap();
        });
    }

    pub fn setup_gateway(&self, gateway: IpAddress) {
        let mut iface = self.iface.lock();
        match gateway {
            IpAddress::Ipv4(v4) => iface.routes_mut().add_default_ipv4_route(v4).unwrap(),
            IpAddress::Ipv6(v6) => iface.routes_mut().add_default_ipv6_route(v6).unwrap(),
        };
    }

    pub fn poll(&self, sockets: &Mutex<SocketSet>) {
        let mut dev = self.dev.lock();
        let mut iface = self.iface.lock();
        let mut sockets = sockets.lock();
        let timestamp = Self::current_time();
        iface.poll(timestamp, dev.deref_mut(), &mut sockets);
    }
}

impl DeviceWrapper {
    fn new(inner: AxNetDevice) -> Self {
        Self {
            inner: RefCell::new(inner),
        }
    }
}

impl Device for DeviceWrapper {
    type RxToken<'a>
        = AxNetRxToken<'a>
    where
        Self: 'a;
    type TxToken<'a>
        = AxNetTxToken<'a>
    where
        Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let mut dev = self.inner.borrow_mut();
        if let Err(e) = dev.recycle_tx_buffers() {
            warn!("recycle_tx_buffers failed: {:?}", e);
            return None;
        }

        if !dev.can_transmit() {
            return None;
        }
        let rx_buf = match dev.receive() {
            Ok(buf) => buf,
            Err(err) => {
                if !matches!(err, DevError::Again) {
                    warn!("receive failed: {:?}", err);
                }
                return None;
            }
        };
        Some((AxNetRxToken(&self.inner, rx_buf), AxNetTxToken(&self.inner)))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        let mut dev = self.inner.borrow_mut();
        if let Err(e) = dev.recycle_tx_buffers() {
            warn!("recycle_tx_buffers failed: {:?}", e);
            return None;
        }
        if dev.can_transmit() {
            Some(AxNetTxToken(&self.inner))
        } else {
            None
        }
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1514;
        caps.max_burst_size = None;
        caps.medium = Medium::Ethernet;
        caps
    }
}

struct AxNetRxToken<'a>(&'a RefCell<AxNetDevice>, NetBufPtr);
struct AxNetTxToken<'a>(&'a RefCell<AxNetDevice>);

impl<'a> RxToken for AxNetRxToken<'a> {
    fn preprocess(&self, sockets: &mut SocketSet<'_>) {
        snoop_tcp_packet(self.1.packet(), sockets).ok();
    }

    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        let mut rx_buf = self.1;
        trace!(
            "RECV {} bytes: {:02X?}",
            rx_buf.packet_len(),
            rx_buf.packet()
        );
        let result = f(rx_buf.packet_mut());
        self.0.borrow_mut().recycle_rx_buffer(rx_buf).unwrap();
        result
    }
}

impl<'a> TxToken for AxNetTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut dev = self.0.borrow_mut();
        let mut tx_buf = dev.alloc_tx_buffer(len).unwrap();
        let ret = f(tx_buf.packet_mut());
        trace!("SEND {} bytes: {:02X?}", len, tx_buf.packet());
        dev.transmit(tx_buf).unwrap();
        ret
    }
}

fn snoop_tcp_packet(buf: &[u8], sockets: &mut SocketSet<'_>) -> Result<(), smoltcp::wire::Error> {
    use smoltcp::wire::{EthernetFrame, IpProtocol, Ipv4Packet, TcpPacket};

    let ether_frame = EthernetFrame::new_checked(buf)?;
    let ipv4_packet = Ipv4Packet::new_checked(ether_frame.payload())?;

    if ipv4_packet.next_header() == IpProtocol::Tcp {
        let tcp_packet = TcpPacket::new_checked(ipv4_packet.payload())?;
        let src_addr = (ipv4_packet.src_addr(), tcp_packet.src_port()).into();
        let dst_addr = (ipv4_packet.dst_addr(), tcp_packet.dst_port()).into();
        let is_first = tcp_packet.syn() && !tcp_packet.ack();
        if is_first {
            // create a socket for the first incoming TCP packet, as the later accept()
            // returns.
            LISTEN_TABLE.incoming_tcp_packet(src_addr, dst_addr, sockets);
        }
    }
    Ok(())
}

/// Poll the network stack.
///
/// It may receive packets from the NIC and process them, and transmit queued
/// packets to the NIC.
pub fn poll_interfaces() {
    SOCKET_SET.poll_interfaces();
}

/// Add multicast_addr to the loopback device.
pub fn add_membership(multicast_addr: IpAddress, _interface_addr: IpAddress) {
    let _ = LOOPBACK.lock().join_multicast_group(multicast_addr);
}
