mod dns;
mod listen_table;
mod tcp;
mod udp;

use alloc::{collections::VecDeque, vec};
use core::cell::RefCell;
use core::ops::DerefMut;

use axdriver::prelude::*;
use axhal::time::{current_time_nanos, NANOS_PER_MICROS};
use axsync::Mutex;
use driver_net::{DevError, NetBufferBox, NetBufferPool};
use lazy_init::LazyInit;
use smoltcp::iface::{Config, Interface, SocketHandle, SocketSet};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::socket::{self, AnySocket};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr};

use self::listen_table::ListenTable;

pub use self::dns::resolve_socket_addr;
pub use self::tcp::TcpSocket;
pub use self::udp::UdpSocket;

const IP: IpAddress = IpAddress::v4(10, 0, 2, 15); // QEMU user networking default IP
const GATEWAY: IpAddress = IpAddress::v4(10, 0, 2, 2); // QEMU user networking gateway
const DNS_SEVER: IpAddress = IpAddress::v4(8, 8, 8, 8);
const IP_PREFIX: u8 = 24;

const RANDOM_SEED: u64 = 0xA2CE_05A2_CE05_A2CE;

const TCP_RX_BUF_LEN: usize = 64 * 1024;
const TCP_TX_BUF_LEN: usize = 64 * 1024;
const UDP_RX_BUF_LEN: usize = 64 * 1024;
const UDP_TX_BUF_LEN: usize = 64 * 1024;
const RX_BUF_QUEUE_SIZE: usize = 64;
const LISTEN_QUEUE_SIZE: usize = 512;

const NET_BUF_LEN: usize = 1526;
const NET_BUF_POOL_SIZE: usize = 128;

static NET_BUF_POOL: LazyInit<NetBufferPool> = LazyInit::new();

static LISTEN_TABLE: LazyInit<ListenTable> = LazyInit::new();
static SOCKET_SET: LazyInit<SocketSetWrapper> = LazyInit::new();
static ETH0: LazyInit<InterfaceWrapper> = LazyInit::new();

struct SocketSetWrapper<'a>(Mutex<SocketSet<'a>>);

struct DeviceWrapper {
    inner: RefCell<AxNetDevice>, // use `RefCell` is enough since it's wrapped in `Mutex` in `InterfaceWrapper`.
    rx_buf_queue: VecDeque<NetBufferBox<'static>>,
}

struct InterfaceWrapper {
    name: &'static str,
    ether_addr: EthernetAddress,
    dev: Mutex<DeviceWrapper>,
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

    pub fn new_udp_socket() -> socket::udp::Socket<'a> {
        let udp_rx_buffer = socket::udp::PacketBuffer::new(
            vec![socket::udp::PacketMetadata::EMPTY; 8],
            vec![0; UDP_RX_BUF_LEN],
        );
        let udp_tx_buffer = socket::udp::PacketBuffer::new(
            vec![socket::udp::PacketMetadata::EMPTY; 8],
            vec![0; UDP_TX_BUF_LEN],
        );
        socket::udp::Socket::new(udp_rx_buffer, udp_tx_buffer)
    }

    pub fn new_dns_socket() -> socket::dns::Socket<'a> {
        socket::dns::Socket::new(&[DNS_SEVER], vec![])
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

impl InterfaceWrapper {
    fn new(name: &'static str, dev: AxNetDevice, ether_addr: EthernetAddress) -> Self {
        let mut config = Config::new(HardwareAddress::Ethernet(ether_addr));
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
        Instant::from_micros_const((current_time_nanos() / NANOS_PER_MICROS) as i64)
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
        };
    }

    pub fn poll(&self, sockets: &Mutex<SocketSet>) {
        let mut dev = self.dev.lock();
        dev.poll(|buf| {
            snoop_tcp_packet(buf).ok(); // preprocess TCP packets
        });

        let timestamp = Self::current_time();
        let mut iface = self.iface.lock();
        let mut sockets = sockets.lock();
        iface.poll(timestamp, dev.deref_mut(), &mut sockets);
    }
}

impl DeviceWrapper {
    fn new(inner: AxNetDevice) -> Self {
        Self {
            inner: RefCell::new(inner),
            rx_buf_queue: VecDeque::with_capacity(RX_BUF_QUEUE_SIZE),
        }
    }

    fn poll<F>(&mut self, f: F)
    where
        F: Fn(&[u8]),
    {
        while self.rx_buf_queue.len() < RX_BUF_QUEUE_SIZE {
            match self.inner.borrow_mut().receive() {
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

    fn receive(&mut self) -> Option<NetBufferBox<'static>> {
        self.rx_buf_queue.pop_front()
    }
}

impl Device for DeviceWrapper {
    type RxToken<'a> = AxNetRxToken<'a> where Self: 'a;
    type TxToken<'a> = AxNetTxToken<'a> where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let rx_buf = self.receive()?;
        Some((AxNetRxToken(&self.inner, rx_buf), AxNetTxToken(&self.inner)))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(AxNetTxToken(&self.inner))
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1514;
        caps.max_burst_size = None;
        caps.medium = Medium::Ethernet;
        caps
    }
}

struct AxNetRxToken<'a>(&'a RefCell<AxNetDevice>, NetBufferBox<'static>);
struct AxNetTxToken<'a>(&'a RefCell<AxNetDevice>);

impl<'a> RxToken for AxNetRxToken<'a> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut rx_buf = self.1;
        trace!(
            "RECV {} bytes: {:02X?}",
            rx_buf.packet().len(),
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
        let mut tx_buf = NET_BUF_POOL.alloc().unwrap();
        dev.prepare_tx_buffer(&mut tx_buf, len).unwrap();
        let result = f(tx_buf.packet_mut());
        trace!("SEND {} bytes: {:02X?}", len, tx_buf.packet());
        dev.transmit(&tx_buf).unwrap();
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

pub(crate) fn init(mut net_dev: AxNetDevice) {
    let pool = NetBufferPool::new(NET_BUF_POOL_SIZE, NET_BUF_LEN).unwrap();
    NET_BUF_POOL.init_by(pool);
    net_dev.fill_rx_buffers(&NET_BUF_POOL).unwrap();

    let ether_addr = EthernetAddress(net_dev.mac_address().0);
    let eth0 = InterfaceWrapper::new("eth0", net_dev, ether_addr);
    eth0.setup_ip_addr(IP, IP_PREFIX);
    eth0.setup_gateway(GATEWAY);

    ETH0.init_by(eth0);
    SOCKET_SET.init_by(SocketSetWrapper::new());
    LISTEN_TABLE.init_by(ListenTable::new());

    info!("created net interface {:?}:", ETH0.name());
    info!("  ether:    {}", ETH0.ethernet_address());
    info!("  ip:       {}/{}", IP, IP_PREFIX);
    info!("  gateway:  {}", GATEWAY);
}
