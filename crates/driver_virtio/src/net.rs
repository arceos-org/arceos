use crate::as_dev_err;
use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};
use driver_net::{EthernetAddress, NetBuffer, NetBufferBox, NetBufferPool, NetDriverOps};
use virtio_drivers::{device::net::VirtIONetRaw as InnerDev, transport::Transport, Hal};

/// The VirtIO network device driver.
///
/// `QS` is the VirtIO queue size.
pub struct VirtIoNetDev<'a, H: Hal, T: Transport, const QS: usize> {
    rx_buffers: [Option<NetBufferBox<'a>>; QS],
    inner: InnerDev<H, T, QS>,
}

unsafe impl<H: Hal, T: Transport, const QS: usize> Send for VirtIoNetDev<'_, H, T, QS> {}
unsafe impl<H: Hal, T: Transport, const QS: usize> Sync for VirtIoNetDev<'_, H, T, QS> {}

impl<'a, H: Hal, T: Transport, const QS: usize> VirtIoNetDev<'a, H, T, QS> {
    /// Creates a new driver instance and initializes the device, or returns
    /// an error if any step fails.
    pub fn try_new(transport: T) -> DevResult<Self> {
        const NONE_BUF: Option<NetBufferBox> = None;
        let inner = InnerDev::new(transport).map_err(as_dev_err)?;
        let rx_buffers = [NONE_BUF; QS];
        Ok(Self { rx_buffers, inner })
    }
}

impl<H: Hal, T: Transport, const QS: usize> const BaseDriverOps for VirtIoNetDev<'_, H, T, QS> {
    fn device_name(&self) -> &str {
        "virtio-net"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Net
    }
}

impl<'a, H: Hal, T: Transport, const QS: usize> NetDriverOps<'a> for VirtIoNetDev<'a, H, T, QS> {
    #[inline]
    fn mac_address(&self) -> EthernetAddress {
        EthernetAddress(self.inner.mac_address())
    }

    #[inline]
    fn can_transmit(&self) -> bool {
        self.inner.can_transmit()
    }

    #[inline]
    fn can_receive(&self) -> bool {
        self.inner.can_receive()
    }

    #[inline]
    fn rx_queue_size(&self) -> usize {
        QS
    }

    #[inline]
    fn tx_queue_size(&self) -> usize {
        QS
    }

    fn fill_rx_buffers(&mut self, buf_pool: &'a NetBufferPool) -> DevResult {
        for (i, rx_buf_place) in self.rx_buffers.iter_mut().enumerate() {
            let mut rx_buf = buf_pool.alloc_boxed().ok_or(DevError::NoMemory)?;
            // Safe because the buffer lives as long as the queue.
            let token = unsafe {
                self.inner
                    .receive_begin(rx_buf.raw_buf_mut())
                    .map_err(as_dev_err)?
            };
            assert_eq!(token, i as u16);
            *rx_buf_place = Some(rx_buf);
        }
        Ok(())
    }

    fn prepare_tx_buffer(&self, tx_buf: &mut NetBuffer, pkt_len: usize) -> DevResult {
        let hdr_len = self
            .inner
            .fill_buffer_header(tx_buf.raw_buf_mut())
            .or(Err(DevError::InvalidParam))?;
        if hdr_len + pkt_len > tx_buf.capacity() {
            return Err(DevError::InvalidParam);
        }
        tx_buf.set_header_len(hdr_len);
        tx_buf.set_packet_len(pkt_len);
        Ok(())
    }

    fn recycle_rx_buffer(&mut self, mut rx_buf: NetBufferBox<'a>) -> DevResult {
        // Safe because we take the ownership of `rx_buf` back to `rx_buffers`,
        // it lives as long as the queue.
        let new_token = unsafe {
            self.inner
                .receive_begin(rx_buf.raw_buf_mut())
                .map_err(as_dev_err)?
        };
        // `rx_buffers[new_token]` is expected to be `None` since it was taken
        // away at `Self::receive()` and has not been added back.
        if self.rx_buffers[new_token as usize].is_some() {
            return Err(DevError::BadState);
        }
        self.rx_buffers[new_token as usize] = Some(rx_buf);
        Ok(())
    }

    fn transmit(&mut self, tx_buf: &NetBuffer) -> DevResult {
        self.inner
            .transmit_wait(tx_buf.packet_with_header())
            .map_err(as_dev_err)?;
        Ok(())
    }

    fn receive(&mut self) -> DevResult<NetBufferBox<'a>> {
        if let Some(token) = self.inner.poll_receive() {
            let mut rx_buf = self.rx_buffers[token as usize]
                .take()
                .ok_or(DevError::BadState)?;
            // Safe because the buffer lives as long as the queue.
            let (hdr_len, pkt_len) = unsafe {
                self.inner
                    .receive_complete(token, rx_buf.raw_buf_mut())
                    .map_err(as_dev_err)?
            };
            rx_buf.set_header_len(hdr_len);
            rx_buf.set_packet_len(pkt_len);
            Ok(rx_buf)
        } else {
            Err(DevError::Again)
        }
    }
}
