use spin::Mutex;

use crate::as_dev_err;
use driver_common::{BaseDriverOps, DevResult, DeviceType};
use driver_net::NetDriverOps;
use virtio_drivers::{device::net::VirtIONet as InnerDev, transport::Transport, Hal};

pub struct VirtIoNetDev<H: Hal, T: Transport> {
    inner: Mutex<InnerDev<H, T>>,
}

unsafe impl<H: Hal, T: Transport> Send for VirtIoNetDev<H, T> {}
unsafe impl<H: Hal, T: Transport> Sync for VirtIoNetDev<H, T> {}

impl<H: Hal, T: Transport> VirtIoNetDev<H, T> {
    pub fn try_new(transport: T) -> DevResult<Self> {
        Ok(Self {
            inner: Mutex::new(InnerDev::new(transport).map_err(as_dev_err)?),
        })
    }
}

impl<H: Hal, T: Transport> const BaseDriverOps for VirtIoNetDev<H, T> {
    fn device_name(&self) -> &str {
        "virtio-net"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Net
    }
}

impl<H: Hal, T: Transport> NetDriverOps for VirtIoNetDev<H, T> {
    fn send(&self, buf: &[u8]) -> DevResult<usize> {
        self.inner.lock().send(buf).map_err(as_dev_err)?;
        Ok(buf.len())
    }

    fn recv(&self, buf: &mut [u8]) -> DevResult<usize> {
        self.inner.lock().recv(buf).map_err(as_dev_err)
    }
}
