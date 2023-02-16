use alloc::{borrow::ToOwned, vec, vec::Vec};
use core::str::FromStr;

use axhal::time::{current_time_nanos, NANOS_PER_MICROS};
use driver_common::DevError;
use driver_net::{NetBuffer, NetDriverOps};
use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address};
use smoltcp::{socket::tcp, time::Instant};
use spin::Mutex;

const IP: &str = "10.0.2.15"; // QEMU user networking default IP
const GATEWAY: &str = "10.0.2.2"; // QEMU user networking gateway
const PORT: u16 = 5555;

struct DeviceWrapper<'a, D: NetDriverOps>(&'a D);

struct AxNetInterface<'a, D: NetDriverOps> {
    _name: &'static str,
    ether_addr: EthernetAddress,
    dev: &'a D,
    iface: Mutex<Interface>,
}

impl<'a, D: NetDriverOps> AxNetInterface<'a, D> {
    fn new(name: &'static str, dev: &'a D) -> Self {
        // Create interface
        let mut config = Config::new();
        let ether_addr = EthernetAddress(dev.mac_address().0);
        config.random_seed = 0x2333;
        config.hardware_addr = Some(HardwareAddress::Ethernet(ether_addr));

        let iface = Mutex::new(Interface::new(config, &mut DeviceWrapper(dev)));
        Self {
            _name: name,
            ether_addr,
            dev,
            iface,
        }
    }

    pub fn ethernet_address(&self) -> EthernetAddress {
        self.ether_addr
    }

    pub fn setup_ip_addrs(&self) {
        let mut iface = self.iface.lock();
        iface.update_ip_addrs(|ip_addrs| {
            ip_addrs
                .push(IpCidr::new(IpAddress::from_str(IP).unwrap(), 24))
                .unwrap();
        });
        iface
            .routes_mut()
            .add_default_ipv4_route(Ipv4Address::from_str(GATEWAY).unwrap())
            .unwrap();
    }

    pub fn poll(&self, sockets: &mut SocketSet) {
        let timestamp =
            Instant::from_micros_const((current_time_nanos() / NANOS_PER_MICROS) as i64);
        self.iface
            .lock()
            .poll(timestamp, &mut DeviceWrapper(self.dev), sockets);
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

pub fn init() {
    let dev = &axdriver::net_devices().0;

    // Create interface
    let iface = AxNetInterface::new("eth0", dev);
    iface.setup_ip_addrs();
    info!("got ethernet address: {}", iface.ethernet_address());

    // Create sockets
    let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
    let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
    let tcp_socket = tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer);

    let mut sockets = SocketSet::new(vec![]);
    let tcp_handle = sockets.add(tcp_socket);

    info!("start a reverse echo server...");
    let mut tcp_active = false;
    loop {
        iface.poll(&mut sockets);

        // tcp:PORT: echo with reverse
        let socket = sockets.get_mut::<tcp::Socket>(tcp_handle);
        if !socket.is_open() {
            info!("listening on port {}...", PORT);
            socket.listen(PORT).unwrap();
        }

        if socket.is_active() && !tcp_active {
            info!("tcp:{} connected", PORT);
        } else if !socket.is_active() && tcp_active {
            info!("tcp:{} disconnected", PORT);
        }
        tcp_active = socket.is_active();

        if socket.may_recv() {
            let data = socket
                .recv(|buffer| {
                    let recvd_len = buffer.len();
                    if !buffer.is_empty() {
                        debug!("tcp:{} recv {} bytes: {:?}", PORT, recvd_len, buffer);
                        let mut lines = buffer
                            .split(|&b| b == b'\n')
                            .map(ToOwned::to_owned)
                            .collect::<Vec<_>>();
                        for line in lines.iter_mut() {
                            line.reverse();
                        }
                        let data = lines.join(&b'\n');
                        (recvd_len, data)
                    } else {
                        (0, vec![])
                    }
                })
                .unwrap();
            if socket.can_send() && !data.is_empty() {
                debug!("tcp:{} send data: {:?}", PORT, data);
                socket.send_slice(&data[..]).unwrap();
            }
        } else if socket.may_send() {
            info!("tcp:{} close", PORT);
            socket.close();
        }
    }
}
