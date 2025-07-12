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
use lazyinit::LazyInit;
use smoltcp::{
    phy::Medium,
    wire::{EthernetAddress, IpAddress, IpCidr},
};
pub use socket::*;
pub use tcp::*;
pub use udp::*;
pub use wrapper::poll_interfaces;

use crate::{
    consts::{IP_PREFIX, RANDOM_SEED},
    listen_table::ListenTable,
    loopback::LoopbackDev,
    wrapper::{DeviceWrapper, InterfaceWrapper, SocketSetWrapper},
};

#[doc(hidden)]
pub mod __priv {
    pub use axerrno::LinuxError;
}

static LISTEN_TABLE: LazyInit<ListenTable> = LazyInit::new();
static SOCKET_SET: LazyInit<SocketSetWrapper> = LazyInit::new();

static LOOPBACK: LazyInit<InterfaceWrapper<LoopbackDev>> = LazyInit::new();
static ETH0: LazyInit<InterfaceWrapper<DeviceWrapper>> = LazyInit::new();

/// Initializes the network subsystem by NIC devices.
pub fn init_network(mut net_devs: AxDeviceContainer<AxNetDevice>) {
    info!("Initialize network subsystem...");

    let dev = net_devs.take_one().expect("No NIC device found!");
    info!("  use NIC 0: {:?}", dev.device_name());

    let device = LoopbackDev::new(Medium::Ip);
    let config = smoltcp::iface::Config::new(smoltcp::wire::HardwareAddress::Ip);
    let iface = InterfaceWrapper::new("lo", device, config);
    iface.inner().lock().update_ip_addrs(|ip_addrs| {
        ip_addrs
            .push(IpCidr::new(IpAddress::v4(127, 0, 0, 1), 8))
            .unwrap();
    });
    LOOPBACK.init_once(iface);

    let ether_addr = EthernetAddress(dev.mac_address().0);
    let mut config =
        smoltcp::iface::Config::new(smoltcp::wire::HardwareAddress::Ethernet(ether_addr));
    // TODO(mivik): random seed
    config.random_seed = RANDOM_SEED;
    let eth0 = InterfaceWrapper::new("eth0", DeviceWrapper::new(dev), config);

    let ip = consts::IP.parse().expect("invalid IP address");
    let gateway = consts::GATEWAY.parse().expect("invalid gateway IP address");
    eth0.setup_ip_addr(ip, consts::IP_PREFIX);
    eth0.setup_gateway(gateway);

    ETH0.init_once(eth0);
    info!("created net interface {:?}:", ETH0.name());
    info!("  ether:    {}", ether_addr);
    info!("  ip:       {}/{}", ip, IP_PREFIX);
    info!("  gateway:  {}", gateway);

    SOCKET_SET.init_once(SocketSetWrapper::new());
    LISTEN_TABLE.init_once(ListenTable::new());
}
