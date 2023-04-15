#![no_std]
#![feature(const_trait_impl)]
#![feature(doc_auto_cfg)]

#[macro_use]
extern crate log;

#[cfg(feature = "block")]
mod blk;
#[cfg(feature = "gpu")]
mod gpu;
#[cfg(feature = "net")]
mod net;

#[cfg(feature = "block")]
pub use self::blk::VirtIoBlkDev;
#[cfg(feature = "gpu")]
pub use self::gpu::VirtIoGpuDev;
#[cfg(feature = "net")]
pub use self::net::VirtIoNetDev;

use driver_common::{DevError, DeviceType};
use virtio_drivers::transport;

pub use virtio_drivers::{transport::Transport, BufferDirection, Hal as VirtIoHal, PhysAddr};

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
        GPU => Some(DeviceType::Display),
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
