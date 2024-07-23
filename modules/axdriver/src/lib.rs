//! [ArceOS](https://github.com/arceos-org/arceos) device drivers.
//!
//! # Usage
//!
//! All detected devices are composed into a large struct [`AllDevices`]
//! and returned by the [`init_drivers`] function. The upperlayer subsystems
//! (e.g., the network stack) may unpack the struct to get the specified device
//! driver they want.
//!
//! For each device category (i.e., net, block, display, etc.), an unified type
//! is used to represent all devices in that category. Currently, there are 3
//! categories: [`AxNetDevice`], [`AxBlockDevice`], and [`AxDisplayDevice`].
//!
//! # Concepts
//!
//! This crate supports two device models depending on the `dyn` feature:
//!
//! - **Static**: The type of all devices is static, it is determined at compile
//!   time by corresponding cargo features. For example, [`AxNetDevice`] will be
//!   an alias of [`VirtioNetDev`] if the `virtio-net` feature is enabled. This
//!   model provides the best performance as it avoids dynamic dispatch. But on
//!   limitation, only one device instance is supported for each device category.
//! - **Dynamic**: All device instance is using [trait objects] and wrapped in a
//!   `Box<dyn Trait>`. For example, [`AxNetDevice`] will be [`Box<dyn NetDriverOps>`].
//!   When call a method provided by the device, it uses [dynamic dispatch][dyn]
//!   that may introduce a little overhead. But on the other hand, it is more
//!   flexible, multiple instances of each device category are supported.
//!
//! # Supported Devices
//!
//! | Device Category | Cargo Feature | Description |
//! |-|-|-|
//! | Block | `ramdisk` | A RAM disk that stores data in a vector |
//! | Block | `virtio-blk` | VirtIO block device |
//! | Network | `virtio-net` | VirtIO network device |
//! | Display | `virtio-gpu` | VirtIO graphics device |
//!
//! # Other Cargo Features
//!
//! - `dyn`: use the dynamic device model (see above).
//! - `bus-mmio`: use device tree to probe all MMIO devices.
//! - `bus-pci`: use PCI bus to probe all PCI devices. This feature is
//!    enabeld by default.
//! - `virtio`: use VirtIO devices. This is enabled if any of `virtio-blk`,
//!   `virtio-net` or `virtio-gpu` is enabled.
//! - `net`: use network devices. This is enabled if any feature of network
//!    devices is selected. If this feature is enabled without any network device
//!    features, a dummy struct is used for [`AxNetDevice`].
//! - `block`: use block storage devices. Similar to the `net` feature.
//! - `display`: use graphics display devices. Similar to the `net` feature.
//!
//! [`VirtioNetDev`]: axdriver_virtio::VirtIoNetDev
//! [`Box<dyn NetDriverOps>`]: axdriver_net::NetDriverOps
//! [trait objects]: https://doc.rust-lang.org/book/ch17-02-trait-objects.html
//! [dyn]: https://doc.rust-lang.org/std/keyword.dyn.html

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(associated_type_defaults)]

#[macro_use]
extern crate log;

#[cfg(feature = "dyn")]
extern crate alloc;

#[macro_use]
mod macros;

mod bus;
mod drivers;
mod dummy;
mod structs;

#[cfg(feature = "virtio")]
mod virtio;

#[cfg(feature = "ixgbe")]
mod ixgbe;

pub mod prelude;

#[allow(unused_imports)]
use self::prelude::*;
pub use self::structs::{AxDeviceContainer, AxDeviceEnum};

#[cfg(feature = "block")]
pub use self::structs::AxBlockDevice;
#[cfg(feature = "display")]
pub use self::structs::AxDisplayDevice;
#[cfg(feature = "net")]
pub use self::structs::AxNetDevice;

/// A structure that contains all device drivers, organized by their category.
#[derive(Default)]
pub struct AllDevices {
    /// All network device drivers.
    #[cfg(feature = "net")]
    pub net: AxDeviceContainer<AxNetDevice>,
    /// All block device drivers.
    #[cfg(feature = "block")]
    pub block: AxDeviceContainer<AxBlockDevice>,
    /// All graphics device drivers.
    #[cfg(feature = "display")]
    pub display: AxDeviceContainer<AxDisplayDevice>,
}

impl AllDevices {
    /// Returns the device model used, either `dyn` or `static`.
    ///
    /// See the [crate-level documentation](crate) for more details.
    pub const fn device_model() -> &'static str {
        if cfg!(feature = "dyn") {
            "dyn"
        } else {
            "static"
        }
    }

    /// Probes all supported devices.
    fn probe(&mut self) {
        for_each_drivers!(type Driver, {
            if let Some(dev) = Driver::probe_global() {
                info!(
                    "registered a new {:?} device: {:?}",
                    dev.device_type(),
                    dev.device_name(),
                );
                self.add_device(dev);
            }
        });

        self.probe_bus_devices();
    }

    /// Adds one device into the corresponding container, according to its device category.
    #[allow(dead_code)]
    fn add_device(&mut self, dev: AxDeviceEnum) {
        match dev {
            #[cfg(feature = "net")]
            AxDeviceEnum::Net(dev) => self.net.push(dev),
            #[cfg(feature = "block")]
            AxDeviceEnum::Block(dev) => self.block.push(dev),
            #[cfg(feature = "display")]
            AxDeviceEnum::Display(dev) => self.display.push(dev),
        }
    }
}

/// Probes and initializes all device drivers, returns the [`AllDevices`] struct.
pub fn init_drivers() -> AllDevices {
    info!("Initialize device drivers...");
    info!("  device model: {}", AllDevices::device_model());

    let mut all_devs = AllDevices::default();
    all_devs.probe();

    #[cfg(feature = "net")]
    {
        debug!("number of NICs: {}", all_devs.net.len());
        for (i, dev) in all_devs.net.iter().enumerate() {
            assert_eq!(dev.device_type(), DeviceType::Net);
            debug!("  NIC {}: {:?}", i, dev.device_name());
        }
    }
    #[cfg(feature = "block")]
    {
        debug!("number of block devices: {}", all_devs.block.len());
        for (i, dev) in all_devs.block.iter().enumerate() {
            assert_eq!(dev.device_type(), DeviceType::Block);
            debug!("  block device {}: {:?}", i, dev.device_name());
        }
    }
    #[cfg(feature = "display")]
    {
        debug!("number of graphics devices: {}", all_devs.display.len());
        for (i, dev) in all_devs.display.iter().enumerate() {
            assert_eq!(dev.device_type(), DeviceType::Display);
            debug!("  graphics device {}: {:?}", i, dev.device_name());
        }
    }

    all_devs
}
