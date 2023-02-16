use spin::Mutex;

use crate::as_dev_err;
use driver_common::{BaseDriverOps, DevResult, DeviceType};
use driver_net::{EthernetAddress, NetBuffer, NetDriverOps};
use virtio_drivers::device::net::{self, VirtIONet as InnerDev};
use virtio_drivers::{transport::Transport, Hal};

pub struct VirtIoNetDev<H: Hal, T: Transport, const QS: usize> {
    inner: Mutex<InnerDev<H, T, QS>>, // TODO: do not use Mutex here
}

unsafe impl<H: Hal, T: Transport, const QS: usize> Send for VirtIoNetDev<H, T, QS> {}
unsafe impl<H: Hal, T: Transport, const QS: usize> Sync for VirtIoNetDev<H, T, QS> {}

pub struct RxBufferWrapper(net::RxBuffer);
pub struct TxBufferWrapper(net::TxBuffer);

impl NetBuffer for RxBufferWrapper {
    #[inline]
    fn packet_len(&self) -> usize {
        self.0.packet_len()
    }

    #[inline]
    fn packet(&self) -> &[u8] {
        self.0.packet()
    }

    #[inline]
    fn packet_mut(&mut self) -> &mut [u8] {
        self.0.packet_mut()
    }
}

impl NetBuffer for TxBufferWrapper {
    #[inline]
    fn packet_len(&self) -> usize {
        self.0.packet_len()
    }

    #[inline]
    fn packet(&self) -> &[u8] {
        self.0.packet()
    }

    #[inline]
    fn packet_mut(&mut self) -> &mut [u8] {
        self.0.packet_mut()
    }
}

impl<H: Hal, T: Transport, const QS: usize> VirtIoNetDev<H, T, QS> {
    pub fn try_new(transport: T, buf_len: usize) -> DevResult<Self> {
        Ok(Self {
            inner: Mutex::new(InnerDev::new(transport, buf_len).map_err(as_dev_err)?),
        })
    }
}

impl<H: Hal, T: Transport, const QS: usize> const BaseDriverOps for VirtIoNetDev<H, T, QS> {
    fn device_name(&self) -> &str {
        "virtio-net"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Net
    }
}

impl<H: Hal, T: Transport, const QS: usize> NetDriverOps for VirtIoNetDev<H, T, QS> {
    type RxBuffer = RxBufferWrapper;
    type TxBuffer = TxBufferWrapper;

    fn mac_address(&self) -> EthernetAddress {
        EthernetAddress(self.inner.lock().mac_address())
    }

    fn can_send(&self) -> bool {
        self.inner.lock().can_send()
    }

    fn can_recv(&self) -> bool {
        self.inner.lock().can_recv()
    }

    fn new_tx_buffer(&self, buf_len: usize) -> DevResult<Self::TxBuffer> {
        Ok(TxBufferWrapper(self.inner.lock().new_tx_buffer(buf_len)))
    }

    fn recycle_rx_buffer(&self, rx_buf: Self::RxBuffer) -> DevResult {
        self.inner
            .lock()
            .recycle_rx_buffer(rx_buf.0)
            .map_err(as_dev_err)?;
        Ok(())
    }

    fn send(&self, tx_buf: Self::TxBuffer) -> DevResult {
        self.inner.lock().send(tx_buf.0).map_err(as_dev_err)?;
        Ok(())
    }

    fn receive(&self) -> DevResult<Self::RxBuffer> {
        Ok(RxBufferWrapper(
            self.inner.lock().receive().map_err(as_dev_err)?,
        ))
    }
}
