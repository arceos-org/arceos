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
//! - [`IpAddr`], [`Ipv4Addr`]: IP addresses (either v4 or v6) and IPv4 addresses.
//! - [`SocketAddr`]: IP address with a port number.
//! - [`resolve_socket_addr`]: Function for DNS query.
//!
//! # Cargo Features
//!
//! - `smoltcp`: Use [smoltcp] as the underlying network stack. This is enabled
//!   by default.
//!
//! [smoltcp]: https://github.com/smoltcp-rs/smoltcp

#![no_std]
#![feature(new_uninit)]
#![feature(const_option)]

#[macro_use]
extern crate log;
extern crate alloc;

cfg_if::cfg_if! {
    if #[cfg(feature = "smoltcp")] {
        mod smoltcp_impl;
        use smoltcp_impl as net_impl;
    }
}

pub use self::net_impl::TcpSocket;
pub use self::net_impl::UdpSocket;
pub use self::net_impl::{bench_receive, bench_transmit};
pub use self::net_impl::{poll_interfaces, resolve_socket_addr};
pub use smoltcp::wire::{IpAddress as IpAddr, IpEndpoint as SocketAddr, Ipv4Address as Ipv4Addr};

use axdriver::{prelude::*, AxDeviceContainer};

/// Initializes the network subsystem by NIC devices.
pub fn init_network(mut net_devs: AxDeviceContainer<AxNetDevice>) {
    info!("Initialize network subsystem...");

    let dev = net_devs.take_one().expect("No NIC device found!");
    info!("  use NIC 0: {:?}", dev.device_name());
    net_impl::init(dev);
}
