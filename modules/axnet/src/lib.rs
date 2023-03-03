#![no_std]
#![feature(new_uninit)]

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
pub use smoltcp::wire::{IpAddress as IpAddr, IpEndpoint as SocketAddr, Ipv4Address as Ipv4Addr};

use axdriver::NetDevices;
use driver_common::{BaseDriverOps, DeviceType};

pub fn init_network(net_devs: NetDevices) {
    info!("Initialize network subsystem...");

    info!("number of NICs: {}", net_devs.len());
    axdriver::net_devices_enumerate!((i, dev) in net_devs {
        assert_eq!(dev.device_type(), DeviceType::Net);
        info!("  NIC {}: {:?}", i, dev.device_name());
    });

    net_impl::init(net_devs);
}
