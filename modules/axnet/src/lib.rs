#![no_std]

#[macro_use]
extern crate log;

use driver_common::{BaseDriverOps, DeviceType};
use driver_net::NetDriverOps;

pub fn init_network() {
    let devices = axdriver::net_devices();
    info!("number of NICs: {}", devices.len());
    axdriver::net_devices_enumerate!((i, dev) in devices {
        assert_eq!(dev.device_type(), DeviceType::Net);
        info!("  NIC {}: {:?}", i, dev.device_name());
    });

    let net_dev = &devices.0;
    let mut buf = [0u8; 0x100];

    info!("Waiting to receive data...");
    let len = net_dev.recv(&mut buf).expect("failed to recv");
    info!("received {} bytes: {:02X?}", len, &buf[..len]);
    net_dev.send(&buf[..len]).expect("failed to send");
    info!("virtio-net test finished.");
}
