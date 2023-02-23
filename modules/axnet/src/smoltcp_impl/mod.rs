mod tcp;

use alloc::vec;
use core::ops::DerefMut;

use axhal::time::{current_time_nanos, NANOS_PER_MICROS};
use driver_common::DevError;
use driver_net::{NetBuffer, NetDriverOps};
use lazy_init::LazyInit;
use smoltcp::iface::{Config, Interface, SocketHandle, SocketSet};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::socket::tcp::{Socket, SocketBuffer};
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr};
use smoltcp::{socket::AnySocket, time::Instant};
use spin::Mutex;

pub use self::tcp::TcpSocket;

const IP: IpAddress = IpAddress::v4(10, 0, 2, 15); // QEMU user networking default IP
const GATEWAY: IpAddress = IpAddress::v4(10, 0, 2, 2); // QEMU user networking gateway
const IP_PREFIX: u8 = 24;

const TCP_RX_BUF_LEN: usize = 4096;
const TCP_TX_BUF_LEN: usize = 4096;

const RANDOM_SEED: u64 = 0xA2CE_05A2_CE05_A2CE;

static SOCKET_SET: LazyInit<SocketSetWrapper> = LazyInit::new();
static ETH0: LazyInit<InterfaceWrapper<DeviceWrapper<'static, axdriver::VirtIoNetDev>>> =
    LazyInit::new();

struct SocketSetWrapper<'a>(Mutex<SocketSet<'a>>);

struct DeviceWrapper<'a, D: NetDriverOps>(&'a D);

struct InterfaceWrapper<D: Device> {
    name: &'static str,
    ether_addr: Option<EthernetAddress>,
    dev: Mutex<D>,
    iface: Mutex<Interface>,
}

impl<'a> SocketSetWrapper<'a> {
    pub fn new_tcp_socket() -> Socket<'a> {
        let tcp_rx_buffer = SocketBuffer::new(vec![0; TCP_RX_BUF_LEN]);
        let tcp_tx_buffer = SocketBuffer::new(vec![0; TCP_TX_BUF_LEN]);
        Socket::new(tcp_rx_buffer, tcp_tx_buffer)
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

impl<D: Device> InterfaceWrapper<D> {
    fn new(name: &'static str, mut dev: D, ether_addr: Option<EthernetAddress>) -> Self {
        let mut config = Config::new();
        config.random_seed = RANDOM_SEED;
        config.hardware_addr = ether_addr.map(HardwareAddress::Ethernet);

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
        let timestamp =
            Instant::from_micros_const((current_time_nanos() / NANOS_PER_MICROS) as i64);
        let mut dev = self.dev.lock();
        let mut iface = self.iface.lock();
        let mut sockets = sockets.lock();
        iface.poll(timestamp, dev.deref_mut(), &mut sockets);
    }
}

impl<'b, D: NetDriverOps> Device for DeviceWrapper<'b, D> {
    type RxToken<'a> = AxNetRxToken<'a, D> where Self: 'a;
    type TxToken<'a> = AxNetTxToken<'a, D> where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        match self.0.receive() {
            Ok(buf) => Some((AxNetRxToken(self.0, buf), AxNetTxToken(self.0))),
            Err(DevError::ResourceBusy) => None, // TODO: better method to check for no data
            Err(err) => {
                warn!("receive failed: {:?}", err);
                None
            }
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(AxNetTxToken(self.0))
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

pub(crate) fn init() {
    let dev = &axdriver::net_devices().0;
    let ether_addr = EthernetAddress(dev.mac_address().0);
    let eth0 = InterfaceWrapper::new("eth0", DeviceWrapper(dev), Some(ether_addr));
    eth0.setup_ip_addr(IP, IP_PREFIX);
    eth0.setup_gateway(GATEWAY);

    ETH0.init_by(eth0);
    SOCKET_SET.init_by(SocketSetWrapper(Mutex::new(SocketSet::new(vec![]))));

    info!("created net interface {:?}:", ETH0.name());
    if let Some(ether_addr) = ETH0.ethernet_address() {
        info!("  ether:    {}", ether_addr);
    }
    info!("  ip:       {}/{}", IP, IP_PREFIX);
    info!("  gateway:  {}", GATEWAY);
}
