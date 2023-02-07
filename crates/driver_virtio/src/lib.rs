#![no_std]

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

use core::{convert::Infallible, marker::PhantomData, ptr::NonNull};

use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::Transport;
use virtio_drivers::Hal;

pub use virtio_drivers::{transport::DeviceType, BufferDirection, Hal as VirtIoHal, PhysAddr};

#[allow(clippy::large_enum_variant)]
pub enum VirtIoDevice<H: Hal, T: Transport> {
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
            match transport.device_type() {
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
