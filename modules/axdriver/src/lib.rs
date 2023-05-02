//! [ArceOS](https://github.com/rcore-os/arceos) device drivers.
//!
//! For each type of supported devices, the drivers are wrapped in a tuple
//! (such as [`BlockDevices`]). You can select the device you want to drive
//! according your platform by cargo features.
//!
//! All types of device drivers are composed into a large struct [`AllDevices`],
//! and returns by the [`init_drivers`] function. The upperlayer subsystems
//! (e.g., the network stack) may unpack the struct to get the specified device
//! driver they want.
//!
//! # Cargo Features
//!
//! - `ramdisk`: use RAM disk block device.
//! - `virtio-blk`: use VirtIO block device.
//! - `virtio-net`: use VirtIO network device.
//! - `virtio-gpu`: use VirtIO GPU device.
//! - `virtio`: use VirtIO devices. This is enabled if any of `virtio-blk`,
//!   `virtio-net` or `virtio-gpu` is enabled.
//! - `bus-mmio`: if `virtio` is enabled, use MMIO bus for VirtIO devices. This
//!    is enabled by default if `virtio` is enabled.
//! - `bus-pci`: if `virtio` is enabled, use PCI bus for VirtIO devices.

#![no_std]
#![feature(doc_auto_cfg)]

#[macro_use]
extern crate log;

#[cfg(feature = "virtio")]
mod virtio;

use tuple_for_each::TupleForEach;

#[cfg(feature = "virtio-blk")]
pub use self::virtio::VirtIoBlockDev;
#[cfg(feature = "virtio-gpu")]
pub use self::virtio::VirtIoGpuDev;
#[cfg(feature = "virtio-net")]
pub use self::virtio::VirtIoNetDev;

/// Alias of [`driver_block::ramdisk::RamDisk`].
#[cfg(feature = "ramdisk")]
pub type RamDisk = driver_block::ramdisk::RamDisk;

/// A tuple of all block device drivers.
#[derive(TupleForEach)]
pub struct BlockDevices(
    #[cfg(feature = "virtio-blk")] pub VirtIoBlockDev,
    #[cfg(feature = "ramdisk")] pub RamDisk,
    // e.g. #[cfg(feature = "nvme")] pub nvme::NVMeDev,
);

/// A tuple of all network device drivers.
#[derive(TupleForEach)]
pub struct NetDevices(
    #[cfg(feature = "virtio-net")] pub VirtIoNetDev,
    // e.g. #[cfg(feature = "e1000")] pub e1000::E1000Dev,
);

/// A tuple of all graphics device drivers.
#[derive(TupleForEach)]
pub struct DisplayDevices(#[cfg(feature = "virtio-gpu")] pub VirtIoGpuDev);

/// A struct that contains all types of device drivers.
pub struct AllDevices {
    /// All block device drivers.
    pub block: BlockDevices,
    /// All network device drivers.
    pub net: NetDevices,
    /// All graphics device drivers.
    pub display: DisplayDevices,
}

impl AllDevices {
    fn probe() -> Self {
        Self {
            block: BlockDevices(
                #[cfg(feature = "virtio-blk")]
                Self::probe_virtio_blk().expect("no virtio-blk device found"),
                #[cfg(feature = "ramdisk")] // TODO: format RAM disk
                RamDisk::new(0x100_0000), // 16 MiB
            ),
            net: NetDevices(
                #[cfg(feature = "virtio-net")]
                Self::probe_virtio_net().expect("no virtio-net device found"),
            ),
            display: DisplayDevices(
                #[cfg(feature = "virtio-gpu")]
                Self::probe_virtio_display().expect("no virtio-gpu device found"),
            ),
        }
    }
}

/// Initialize all device drivers, returns the [`AllDevices`] struct.
///
/// # Panics
///
/// The function panics if the specified device driver (by cargo features) is
/// not found or an error occurs during initialization.
pub fn init_drivers() -> AllDevices {
    info!("Initialize device drivers...");

    AllDevices::probe()
}
