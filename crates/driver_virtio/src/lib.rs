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

use core::{convert::Infallible, marker::PhantomData, ptr::NonNull};

use driver_common::DevError;
use virtio_drivers::transport::{mmio::MmioTransport, mmio::VirtIOHeader, Transport};
use virtio_drivers::Hal;

pub use virtio_drivers::{BufferDirection, Hal as VirtIoHal, PhysAddr};

#[allow(clippy::large_enum_variant)]
pub enum VirtIoDevice<H: Hal, T: Transport> {
    #[cfg(feature = "block")]
    Block(VirtIoBlkDev<H, T>),
    #[cfg(feature = "net")]
    Net(VirtIoNetDev<H, T>),
    _Unknown(Infallible, PhantomData<(H, T)>),
}

pub fn new_from_mmio<H: Hal>(
    reg_base: *mut u8,
    _reg_size: usize,
) -> Option<VirtIoDevice<H, MmioTransport>> {
    let header = NonNull::new(reg_base as *mut VirtIOHeader).unwrap();
    match unsafe { MmioTransport::new(header) } {
        Ok(transport) => {
            info!(
                "Detected virtio MMIO device with vendor id: {:#X}, device type: {:?}, version: {:?}",
                transport.vendor_id(),
                transport.device_type(),
                transport.version(),
            );
            #[allow(unused_imports)]
            use virtio_drivers::transport::DeviceType;
            match transport.device_type() {
                #[cfg(feature = "block")]
                DeviceType::Block => {
                    Some(VirtIoDevice::Block(VirtIoBlkDev::try_new(transport).ok()?))
                }
                #[cfg(feature = "net")]
                DeviceType::Network => {
                    Some(VirtIoDevice::Net(VirtIoNetDev::try_new(transport).ok()?))
                }
                t => {
                    debug!("Unsupported virtio device: {:?}", t);
                    None
                }
            }
        }
        Err(_) => None,
    }
}

#[allow(dead_code)]
const fn as_dev_err(e: virtio_drivers::Error) -> DevError {
    use virtio_drivers::Error::*;
    match e {
        QueueFull => DevError::BadState,
        NotReady => DevError::ResourceBusy,
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
