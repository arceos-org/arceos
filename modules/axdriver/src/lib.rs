#![no_std]
#![allow(unused_imports)]

#[cfg(feature = "virtio")]
mod virtio;

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use lazy_init::LazyInit;

static DEVICES: LazyInit<AllDevices> = LazyInit::new();

struct AllDevices {
    #[cfg(feature = "net")]
    net: Vec<Box<dyn driver_net::NetDriverOps>>,
}

impl AllDevices {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "net")]
            net: Vec::new(),
        }
    }

    pub fn probe(&mut self) {
        #[cfg(feature = "virtio")]
        self.prob_virtio_devices();
    }
}

#[cfg(feature = "net")]
pub fn net_devices() -> &'static Vec<Box<dyn driver_net::NetDriverOps>> {
    &DEVICES.net
}

pub fn init_drivers() {
    let mut all_devices = AllDevices::new();
    all_devices.probe();
    DEVICES.init_by(all_devices);

    #[cfg(feature = "net")]
    {
        let net_dev = DEVICES.net.first().unwrap();
        let mut buf = [0u8; 0x100];

        info!("Waiting to receive data...");
        let len = net_dev.recv(&mut buf).expect("failed to recv");
        info!("received {} bytes: {:02X?}", len, &buf[..len]);
        net_dev.send(&buf[..len]).expect("failed to send");
        info!("virtio-net test finished.");
    }
}
