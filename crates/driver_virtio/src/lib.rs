#![no_std]
#![feature(const_trait_impl)]

#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate log;

cfg_if! {
    if #[cfg(feature = "net")] {
        mod net;
        pub use net::VirtIoNetDev;
    }
}
cfg_if! {
    if #[cfg(feature = "block")] {
        mod blk;
        pub use blk::VirtIoBlkDev;
    }
}

use driver_common::{DevError, DeviceType};
use virtio_drivers::transport::{self, Transport};

pub use virtio_drivers::{BufferDirection, Hal as VirtIoHal, PhysAddr};

#[cfg(feature = "bus-mmio")]
pub use transport::mmio::MmioTransport;
#[cfg(feature = "bus-pci")]
pub use transport::pci::PciTransport;

#[cfg(feature = "bus-mmio")]
pub fn probe_mmio_device(
    reg_base: *mut u8,
    _reg_size: usize,
    type_match: Option<DeviceType>,
) -> Option<MmioTransport> {
    use core::ptr::NonNull;
    use transport::mmio::VirtIOHeader;

    let header = NonNull::new(reg_base as *mut VirtIOHeader).unwrap();
    if let Ok(transport) = unsafe { MmioTransport::new(header) } {
        if type_match.is_none() || as_dev_type(transport.device_type()) == type_match {
            debug!(
                "Detected virtio MMIO device with vendor id: {:#X}, device type: {:?}, version: {:?}",
                transport.vendor_id(),
                transport.device_type(),
                transport.version(),
            );
            Some(transport)
        } else {
            None
        }
    } else {
        None
    }
}

const fn as_dev_type(t: transport::DeviceType) -> Option<DeviceType> {
    use transport::DeviceType::*;
    match t {
        Block => Some(DeviceType::Block),
        Network => Some(DeviceType::Net),
        _ => None,
    }
}

#[allow(dead_code)]
const fn as_dev_err(e: virtio_drivers::Error) -> DevError {
    use virtio_drivers::Error::*;
    match e {
        QueueFull => DevError::BadState,
        NotReady => DevError::Again,
        WrongToken => DevError::BadState,
        AlreadyUsed => DevError::AlreadyExists,
        InvalidParam => DevError::InvalidParam,
        DmaError => DevError::NoMemory,
        IoError => DevError::Io,
        Unsupported => DevError::Unsupported,
        ConfigSpaceTooSmall => DevError::BadState,
        ConfigSpaceMissing => DevError::BadState,
    }
}
