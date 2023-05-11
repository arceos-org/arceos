//! [ArceOS](https://github.com/rcore-os/arceos) device drivers.
//!
//! Currently, all device drivers used are selected by cargo features at compile
//! time, and only one instance of each category of device is supported. For one
//! category of device, the type of its unique instance is **static** to avoid
//! performance overheads (rather than using `dyn Trait`). For example,
//! [`AxNetDevice`] will be an alias of [`VirtioNetDev`] if the specified
//! feature is enabled.
//!
//! # Usage
//!
//! All detected devices are composed into a large struct [`AllDevices`]
//! and returned by the [`init_drivers`] function. The upperlayer subsystems
//! (e.g., the network stack) may unpack the struct to get the specified device
//! driver they want.
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
//! - `bus-mmio`: use device tree to probe all MMIO devices. This feature is
//!    enabeld by default.
//! - `bus-pci`: use PCI bus to probe all PCI devices.
//! - `virtio`: use VirtIO devices. This is enabled if any of `virtio-blk`,
//!   `virtio-net` or `virtio-gpu` is enabled.
//! - `net`: use network devices. This is enabled if any feature of network
//!    devices is selected. If this feature is enabled without any network device
//!    features, a dummy struct is used for [`AxNetDevice`].
//! - `block`: use block storage devices. Similar to the `net` feature.
//! - `display`: use graphics display devices. Similar to the `net` feature.
//!
//! [`VirtioNetDev`]: driver_virtio::VirtIoNetDev

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(associated_type_defaults)]

#[macro_use]
extern crate log;

#[macro_use]
mod macros;

mod drivers;
mod dummy;

#[cfg(feature = "virtio")]
mod virtio;

#[allow(unused_imports)]
use driver_common::{BaseDriverOps, DeviceType};

#[cfg(feature = "block")]
pub use self::drivers::AxBlockDevice;
#[cfg(feature = "display")]
pub use self::drivers::AxDisplayDevice;
#[cfg(feature = "net")]
pub use self::drivers::AxNetDevice;

/// A unified enum that represents different categories of devices.
#[allow(clippy::large_enum_variant)]
pub enum AxDeviceEnum {
    #[cfg(feature = "net")]
    Net(AxNetDevice),
    #[cfg(feature = "block")]
    Block(AxBlockDevice),
    #[cfg(feature = "display")]
    Display(AxDisplayDevice),
}

impl BaseDriverOps for AxDeviceEnum {
    #[inline]
    fn device_type(&self) -> DeviceType {
        match self {
            #[cfg(feature = "net")]
            Self::Net(_) => DeviceType::Net,
            #[cfg(feature = "block")]
            Self::Block(_) => DeviceType::Block,
            #[cfg(feature = "display")]
            Self::Display(_) => DeviceType::Display,
        }
    }

    #[inline]
    fn device_name(&self) -> &str {
        match self {
            #[cfg(feature = "net")]
            Self::Net(dev) => dev.device_name(),
            #[cfg(feature = "block")]
            Self::Block(dev) => dev.device_name(),
            #[cfg(feature = "display")]
            Self::Display(dev) => dev.device_name(),
        }
    }
}

/// A structure that contains all device drivers of a certain category.
///
/// Currently, the inner type is [`Option<D>`] and at most one device can be contained.
pub struct AxDeviceContainer<D: BaseDriverOps>(Option<D>);

impl<D: BaseDriverOps> AxDeviceContainer<D> {
    /// Returns number of devices in this container.
    pub const fn len(&self) -> usize {
        if self.0.is_some() {
            1
        } else {
            0
        }
    }

    /// Returns whether the container is empty.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Takes one device out of the container (will remove it from the container).
    pub fn take_one(&mut self) -> Option<D> {
        self.0.take()
    }

    /// Constructs the container from one device.
    pub const fn from_one(dev: D) -> Self {
        Self(Some(dev))
    }

    /// Adds one device into the container.
    #[allow(dead_code)]
    pub(crate) fn push(&mut self, dev: D) {
        if self.0.is_none() {
            self.0 = Some(dev);
        }
    }
}

impl<D: BaseDriverOps> core::ops::Deref for AxDeviceContainer<D> {
    type Target = Option<D>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D: BaseDriverOps> Default for AxDeviceContainer<D> {
    fn default() -> Self {
        Self(Default::default())
    }
}

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

        #[cfg(feature = "virtio")]
        self.probe_virtio_devices();
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
