//! [ArceOS](https://github.com/rcore-os/arceos) network module.
//!
//! It provides unified networking primitives for TCP/UDP communication
//! using various underlying network stacks. Currently, only [smoltcp] is
//! supported.
//!
//! # Organization
//!
//! - [`TcpSocket`]: A TCP socket that provides POSIX-like APIs.
//! - [`UdpSocket`]: A UDP socket that provides POSIX-like APIs.
//! - [`dns_query`]: Function for DNS query.
//!
//! [smoltcp]: https://github.com/smoltcp-rs/smoltcp

#![no_std]
#![feature(ip_from)]

#[macro_use]
extern crate log;
extern crate alloc;

mod consts;
mod general;
mod listen_table;
mod loopback;
pub mod options;
mod socket;
mod tcp;
mod udp;
mod wrapper;

use axdriver::{AxDeviceContainer, prelude::*};
use axsync::Mutex;
use lazyinit::LazyInit;
use smoltcp::{
    iface::Interface,
    phy::Medium,
    time::Instant,
    wire::{EthernetAddress, IpAddress, IpCidr},
};
pub use socket::*;
pub use tcp::*;
pub use udp::*;
pub use wrapper::poll_interfaces;

use crate::{
    consts::IP_PREFIX,
    listen_table::ListenTable,
    loopback::LoopbackDev,
    wrapper::{InterfaceWrapper, SocketSetWrapper},
};

#[doc(hidden)]
pub mod __priv {
    pub use axerrno::LinuxError;
}

static LISTEN_TABLE: LazyInit<ListenTable> = LazyInit::new();
static SOCKET_SET: LazyInit<SocketSetWrapper> = LazyInit::new();

static LOOPBACK_DEV: LazyInit<Mutex<LoopbackDev>> = LazyInit::new();
static LOOPBACK: LazyInit<Mutex<Interface>> = LazyInit::new();

static ETH0: LazyInit<InterfaceWrapper> = LazyInit::new();

/// Initializes the network subsystem by NIC devices.
pub fn init_network(mut net_devs: AxDeviceContainer<AxNetDevice>) {
    info!("Initialize network subsystem...");

    let dev = net_devs.take_one().expect("No NIC device found!");
    info!("  use NIC 0: {:?}", dev.device_name());

    let mut device = LoopbackDev::new(Medium::Ip);
    let config = smoltcp::iface::Config::new(smoltcp::wire::HardwareAddress::Ip);

    let mut iface = Interface::new(config, &mut device, Instant::from_micros_const(0));
    iface.update_ip_addrs(|ip_addrs| {
        ip_addrs
            .push(IpCidr::new(IpAddress::v4(127, 0, 0, 1), 8))
            .unwrap();
    });
    LOOPBACK.init_once(Mutex::new(iface));
    LOOPBACK_DEV.init_once(Mutex::new(device));

    let ether_addr = EthernetAddress(dev.mac_address().0);
    let eth0 = InterfaceWrapper::new("eth0", dev, ether_addr);

    let ip = consts::IP.parse().expect("invalid IP address");
    let gateway = consts::GATEWAY.parse().expect("invalid gateway IP address");
    eth0.setup_ip_addr(ip, consts::IP_PREFIX);
    eth0.setup_gateway(gateway);

    ETH0.init_once(eth0);
    info!("created net interface {:?}:", ETH0.name());
    info!("  ether:    {}", ETH0.ethernet_address());
    info!("  ip:       {}/{}", ip, IP_PREFIX);
    info!("  gateway:  {}", gateway);

    SOCKET_SET.init_once(SocketSetWrapper::new());
    LISTEN_TABLE.init_once(ListenTable::new());
}
