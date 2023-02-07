use spin::Mutex;

use driver_net::{NetDevError, NetDevResult, NetDriverOps};
use virtio_drivers::{device::net::VirtIONet as InnerDev, transport::Transport, Hal};

pub struct VirtIoNetDev<H: Hal, T: Transport> {
    inner: Mutex<InnerDev<H, T>>,
}

unsafe impl<H: Hal, T: Transport> Send for VirtIoNetDev<H, T> {}
unsafe impl<H: Hal, T: Transport> Sync for VirtIoNetDev<H, T> {}

impl<H: Hal, T: Transport> VirtIoNetDev<H, T> {
    pub fn try_new(transport: T) -> NetDevResult<Self> {
        Ok(Self {
            inner: Mutex::new(InnerDev::new(transport).map_err(as_netdev_err)?),
        })
    }
}

impl<H: Hal, T: Transport> NetDriverOps for VirtIoNetDev<H, T> {
    fn send(&self, buf: &[u8]) -> NetDevResult<usize> {
        self.inner.lock().send(buf).map_err(as_netdev_err)?;
        Ok(buf.len())
    }

    fn recv(&self, buf: &mut [u8]) -> NetDevResult<usize> {
        self.inner.lock().recv(buf).map_err(as_netdev_err)
    }
}

const fn as_netdev_err(_e: virtio_drivers::Error) -> NetDevError {
    NetDevError::Dummy
}
