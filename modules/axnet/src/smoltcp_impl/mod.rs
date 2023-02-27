mod listen_table;
mod tcp;

use alloc::{collections::VecDeque, vec};
use core::ops::DerefMut;

use axhal::time::{current_time_nanos, NANOS_PER_MICROS};
use driver_common::DevError;
use driver_net::{NetBuffer, NetDriverOps};
use lazy_init::LazyInit;
use smoltcp::iface::{Config, Interface, SocketHandle, SocketSet};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::socket::{self, AnySocket};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr};
use spin::Mutex;

use self::listen_table::ListenTable;

pub use self::tcp::TcpSocket;

const IP: IpAddress = IpAddress::v4(10, 0, 2, 15); // QEMU user networking default IP
const GATEWAY: IpAddress = IpAddress::v4(10, 0, 2, 2); // QEMU user networking gateway
const IP_PREFIX: u8 = 24;

const RANDOM_SEED: u64 = 0xA2CE_05A2_CE05_A2CE;

const TCP_RX_BUF_LEN: usize = 4096;
const TCP_TX_BUF_LEN: usize = 4096;

const RX_BUF_QUEUE_SIZE: usize = 64;
const LISTEN_QUEUE_SIZE: usize = 512;

static LISTEN_TABLE: LazyInit<ListenTable> = LazyInit::new();
static SOCKET_SET: LazyInit<SocketSetWrapper> = LazyInit::new();
static ETH0: LazyInit<InterfaceWrapper<'static, axdriver::VirtIoNetDev>> = LazyInit::new();

struct SocketSetWrapper<'a>(Mutex<SocketSet<'a>>);

struct DeviceWrapper<'a, D: NetDriverOps> {
    inner: &'a D,
    rx_buf_queue: VecDeque<D::RxBuffer>,
}

struct InterfaceWrapper<'a, D: NetDriverOps> {
    name: &'static str,
    ether_addr: Option<EthernetAddress>,
    dev: Mutex<DeviceWrapper<'a, D>>,
    iface: Mutex<Interface>,
}

impl<'a> SocketSetWrapper<'a> {
    fn new() -> Self {
        Self(Mutex::new(SocketSet::new(vec![])))
    }

    pub fn new_tcp_socket() -> socket::tcp::Socket<'a> {
        let tcp_rx_buffer = socket::tcp::SocketBuffer::new(vec![0; TCP_RX_BUF_LEN]);
        let tcp_tx_buffer = socket::tcp::SocketBuffer::new(vec![0; TCP_TX_BUF_LEN]);
        socket::tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer)
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

    pub fn poll_interfaces(&self) {
        ETH0.poll(&self.0);
    }

    pub fn remove(&self, handle: SocketHandle) {
        self.0.lock().remove(handle);
        debug!("socket {}: destroyed", handle);
    }
}

impl<'a, D: NetDriverOps> InterfaceWrapper<'a, D> {
    fn new(name: &'static str, dev: &'a D, ether_addr: Option<EthernetAddress>) -> Self {
        let mut config = Config::new();
        config.random_seed = RANDOM_SEED;
        config.hardware_addr = ether_addr.map(HardwareAddress::Ethernet);

        let mut dev = DeviceWrapper::new(dev);
        let iface = Mutex::new(Interface::new(config, &mut dev));
        Self {
            name,
            ether_addr,
            dev: Mutex::new(dev),
            iface,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn ethernet_address(&self) -> Option<EthernetAddress> {
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
        };
    }

    pub fn poll(&self, sockets: &Mutex<SocketSet>) {
        let mut dev = self.dev.lock();
        dev.poll(|buf| {
            snoop_tcp_packet(buf).ok(); // preprocess TCP packets
        });

        let timestamp =
            Instant::from_micros_const((current_time_nanos() / NANOS_PER_MICROS) as i64);
        let mut iface = self.iface.lock();
        let mut sockets = sockets.lock();
        iface.poll(timestamp, dev.deref_mut(), &mut sockets);
    }
}

impl<'a, D: NetDriverOps> DeviceWrapper<'a, D> {
    fn new(inner: &'a D) -> Self {
        Self {
            inner,
            rx_buf_queue: VecDeque::with_capacity(RX_BUF_QUEUE_SIZE),
        }
    }

    fn poll<F>(&mut self, f: F)
    where
        F: Fn(&[u8]),
    {
        while self.rx_buf_queue.len() < RX_BUF_QUEUE_SIZE {
            match self.inner.receive() {
                Ok(buf) => {
                    f(buf.packet());
                    self.rx_buf_queue.push_back(buf);
                }
                Err(DevError::Again) => break, // TODO: better method to avoid error type conversion
                Err(err) => {
                    warn!("receive failed: {:?}", err);
                    break;
                }
            }
        }
    }

    fn receive(&mut self) -> Option<D::RxBuffer> {
        self.rx_buf_queue.pop_front()
    }
}

impl<'b, D: NetDriverOps> Device for DeviceWrapper<'b, D> {
    type RxToken<'a> = AxNetRxToken<'a, D> where Self: 'a;
    type TxToken<'a> = AxNetTxToken<'a, D> where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if let Some(buf) = self.receive() {
            Some((AxNetRxToken(self.inner, buf), AxNetTxToken(self.inner)))
        } else {
            None
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(AxNetTxToken(self.inner))
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1536;
        caps.max_burst_size = Some(1);
        caps.medium = Medium::Ethernet;
        caps
    }
}

struct AxNetRxToken<'a, D: NetDriverOps>(&'a D, D::RxBuffer);
struct AxNetTxToken<'a, D: NetDriverOps>(&'a D);

impl<'a, D: NetDriverOps> RxToken for AxNetRxToken<'a, D> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut rx_buf = self.1;
        trace!(
            "RECV {} bytes: {:02X?}",
            rx_buf.packet_len(),
            rx_buf.packet()
        );
        let result = f(rx_buf.packet_mut());
        self.0.recycle_rx_buffer(rx_buf).unwrap();
        result
    }
}

impl<'a, D: NetDriverOps> TxToken for AxNetTxToken<'a, D> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut tx_buf = self.0.new_tx_buffer(len).unwrap();
        let result = f(tx_buf.packet_mut());
        trace!("SEND {} bytes: {:02X?}", len, tx_buf.packet());
        self.0.send(tx_buf).unwrap();
        result
    }
}

fn snoop_tcp_packet(buf: &[u8]) -> Result<(), smoltcp::wire::Error> {
    use crate::SocketAddr;
    use smoltcp::wire::{EthernetFrame, IpProtocol, Ipv4Packet, TcpPacket};

    let ether_frame = EthernetFrame::new_checked(buf)?;
    let ipv4_packet = Ipv4Packet::new_checked(ether_frame.payload())?;

    if ipv4_packet.next_header() == IpProtocol::Tcp {
        let tcp_packet = TcpPacket::new_checked(ipv4_packet.payload())?;
        let src_addr = SocketAddr::new(ipv4_packet.src_addr().into(), tcp_packet.src_port());
        let dst_addr = SocketAddr::new(ipv4_packet.dst_addr().into(), tcp_packet.dst_port());
        let is_first = tcp_packet.syn() && !tcp_packet.ack();
        if is_first {
            // create a socket for the first incoming TCP packet, as the later accept() returns.
            LISTEN_TABLE.incoming_tcp_packet(src_addr, dst_addr);
        }
    }
    Ok(())
}

pub(crate) fn init() {
    let dev = &axdriver::net_devices().0;
    let ether_addr = EthernetAddress(dev.mac_address().0);
    let eth0 = InterfaceWrapper::new("eth0", dev, Some(ether_addr));
    eth0.setup_ip_addr(IP, IP_PREFIX);
    eth0.setup_gateway(GATEWAY);

    ETH0.init_by(eth0);
    SOCKET_SET.init_by(SocketSetWrapper::new());
    LISTEN_TABLE.init_by(ListenTable::new());

    info!("created net interface {:?}:", ETH0.name());
    if let Some(ether_addr) = ETH0.ethernet_address() {
        info!("  ether:    {}", ether_addr);
    }
    info!("  ip:       {}/{}", IP, IP_PREFIX);
    info!("  gateway:  {}", GATEWAY);
}
