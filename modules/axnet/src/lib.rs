#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

#[cfg(feature = "smoltcp")]
mod smoltcp_wrapper;

#[cfg(feature = "smoltcp")]
use smoltcp_wrapper as wrapper;

use driver_common::{BaseDriverOps, DeviceType};

pub fn init_network() {
    let devices = axdriver::net_devices();
    info!("number of NICs: {}", devices.len());
    axdriver::net_devices_enumerate!((i, dev) in devices {
        assert_eq!(dev.device_type(), DeviceType::Net);
        info!("  NIC {}: {:?}", i, dev.device_name());
    });

    wrapper::init();
}
