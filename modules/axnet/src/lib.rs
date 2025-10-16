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
#![feature(maybe_uninit_slice)]

#[macro_use]
extern crate log;
extern crate alloc;

mod consts;
mod device;
mod general;
mod listen_table;
pub mod options;
mod router;
mod service;
mod socket;
pub(crate) mod state;
pub mod tcp;
pub mod udp;
pub mod unix;
#[cfg(feature = "vsock")]
pub mod vsock;
mod wrapper;

use alloc::{borrow::ToOwned, boxed::Box};

use axdriver::{AxDeviceContainer, prelude::*};
use axsync::Mutex;
use lazyinit::LazyInit;
use smoltcp::wire::{EthernetAddress, Ipv4Address, Ipv4Cidr};
pub use socket::*;

use crate::{
    consts::{GATEWAY, IP, IP_PREFIX},
    device::{EthernetDevice, LoopbackDevice},
    listen_table::ListenTable,
    router::{Router, Rule},
    service::Service,
    wrapper::SocketSetWrapper,
};


static LISTEN_TABLE: LazyInit<ListenTable> = LazyInit::new();
static SOCKET_SET: LazyInit<SocketSetWrapper> = LazyInit::new();

static SERVICE: LazyInit<Mutex<Service>> = LazyInit::new();

/// Initializes the network subsystem by NIC devices.
pub fn init_network(mut net_devs: AxDeviceContainer<AxNetDevice>) {
    info!("Initialize network subsystem...");

    let mut router = Router::new();
    let lo_dev = router.add_device(Box::new(LoopbackDevice::new()));

    let lo_ip = Ipv4Cidr::new(Ipv4Address::new(127, 0, 0, 1), 8);
    router.add_rule(Rule::new(
        lo_ip.into(),
        None,
        lo_dev,
        lo_ip.address().into(),
    ));

    let eth0_ip = if let Some(dev) = net_devs.take_one() {
        info!("  use NIC 0: {:?}", dev.device_name());

        let eth0_address = EthernetAddress(dev.mac_address().0);
        let eth0_ip = Ipv4Cidr::new(IP.parse().expect("Invalid IPv4 address"), IP_PREFIX);

        let eth0_dev = router.add_device(Box::new(EthernetDevice::new(
            "eth0".to_owned(),
            dev,
            eth0_ip,
        )));

        router.add_rule(Rule::new(
            Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0).into(),
            Some(GATEWAY.parse().expect("Invalid gateway address")),
            eth0_dev,
            eth0_ip.address().into(),
        ));

        info!("eth0:");
        info!("  mac:  {}", eth0_address);
        info!("  ip:   {}", eth0_ip);

        Some(eth0_ip)
    } else {
        warn!("No NIC device found!");
        None
    };

    let mut service = Service::new(router);
    service.iface.update_ip_addrs(|ip_addrs| {
        ip_addrs.push(lo_ip.into()).unwrap();
        if let Some(eth0_ip) = eth0_ip {
            ip_addrs.push(eth0_ip.into()).unwrap();
        }
    });
    SERVICE.init_once(Mutex::new(service));

    SOCKET_SET.init_once(SocketSetWrapper::new());
    LISTEN_TABLE.init_once(ListenTable::new());
}

/// Init vsock subsystem by vsock devices.
#[cfg(feature = "vsock")]
pub fn init_vsock(mut socket_devs: AxDeviceContainer<AxVsockDevice>) {
    use crate::device::register_vsock_device;
    info!("Initialize vsock subsystem...");
    if let Some(dev) = socket_devs.take_one() {
        info!("  use vsock 0: {:?}", dev.device_name());
        if let Err(e) = register_vsock_device(dev) {
            warn!("Failed to initialize vsock device: {:?}", e);
        }
    } else {
        info!("No vsock device found!");
    }
}

pub fn poll_interfaces() {
    while SERVICE.lock().poll(&mut SOCKET_SET.inner.lock()) {}
}
