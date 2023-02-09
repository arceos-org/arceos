#![no_std]
#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(feature = "virtio")]
mod virtio;

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use driver_common::BaseDriverOps;
use lazy_init::LazyInit;

static DEVICES: LazyInit<AllDevices> = LazyInit::new();

pub struct DeviceList<T: BaseDriverOps + ?Sized>(Vec<Box<T>>);

struct AllDevices {
    #[cfg(feature = "block")]
    block: DeviceList<dyn driver_block::BlockDriverOps>,
    #[cfg(feature = "net")]
    net: DeviceList<dyn driver_net::NetDriverOps>,
}

impl<T: BaseDriverOps + ?Sized> DeviceList<T> {
    pub(crate) const fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn add(&mut self, dev: Box<T>) {
        info!(
            "Added new {:?} device: {:?}",
            dev.device_type(),
            dev.device_name()
        );
        self.0.push(dev);
    }

    /// Returns the device at given position, or `None` if out of bounds.
    #[inline]
    pub fn try_get(&self, idx: usize) -> Option<&T> {
        self.0.get(idx).map(Box::as_ref)
    }

    /// Returns the device with the given name, or `None` if not found.
    pub fn find(&self, name: &str) -> Option<&T> {
        self.0
            .iter()
            .find(|d| d.device_name() == name)
            .map(Box::as_ref)
    }

    /// Returns the first device of this device array, or `None` if it is empty.
    #[inline]
    pub fn first(&self) -> Option<&T> {
        self.try_get(0)
    }
}

impl AllDevices {
    pub const fn new() -> Self {
        Self {
            #[cfg(feature = "block")]
            block: DeviceList::new(),
            #[cfg(feature = "net")]
            net: DeviceList::new(),
        }
    }

    pub fn probe(&mut self) {
        #[cfg(feature = "virtio")]
        self.prob_virtio_devices();
    }
}

#[cfg(feature = "block")]
pub fn block_devices() -> &'static DeviceList<dyn driver_block::BlockDriverOps> {
    &DEVICES.block
}

#[cfg(feature = "net")]
pub fn net_devices() -> &'static DeviceList<dyn driver_net::NetDriverOps> {
    &DEVICES.net
}

pub fn init_drivers() {
    let mut all_devices = AllDevices::new();
    all_devices.probe();
    DEVICES.init_by(all_devices);

    #[cfg(feature = "net")]
    {
        let net_dev = DEVICES.net.first().expect("NIC not found");
        let mut buf = [0u8; 0x100];

        info!("Waiting to receive data...");
        let len = net_dev.recv(&mut buf).expect("failed to recv");
        info!("received {} bytes: {:02X?}", len, &buf[..len]);
        net_dev.send(&buf[..len]).expect("failed to send");
        info!("virtio-net test finished.");
    }
}
