//! [ArceOS](https://github.com/arceos-org/arceos) network module.
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
//! # Cargo Features
//!
//! - `smoltcp`: Use [smoltcp] as the underlying network stack. This is enabled
//!   by default.
//!
//! [smoltcp]: https://github.com/smoltcp-rs/smoltcp

#![no_std]

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
pub use self::net_impl::{dns_query, poll_interfaces};

use axdriver::{AxDeviceContainer, prelude::*};

/// Initializes the network subsystem by NIC devices.
pub fn init_network(mut net_devs: AxDeviceContainer<AxNetDevice>) {
    info!("Initialize network subsystem...");

    if let Some(dev) = net_devs.take_one() {
        info!("  use NIC 0: {:?}", dev.device_name());
        net_impl::init(dev);
    } else {
        warn!("  No network device found!");
    }
}

/// Initializes vsock devices.
#[cfg(feature = "vsock")]
pub fn init_vsock(mut vsock_devs: AxDeviceContainer<AxVsockDevice>) {
    info!("Initialize vsock subsystem...");
    if let Some(dev) = vsock_devs.take_one() {
        info!("  use vsock 0: {:?}", dev.device_name());
        warn!("  vsock not implemented yet!");
    } else {
        warn!("  No vsock device found!");
    }
}
