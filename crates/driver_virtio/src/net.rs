use crate::as_dev_err;
use alloc::{sync::Arc, vec::Vec};
use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};
use driver_net::{EthernetAddress, NetBuf, NetBufBox, NetBufPool, NetBufPtr, NetDriverOps};
use virtio_drivers::{device::net::VirtIONetRaw as InnerDev, transport::Transport, Hal};

extern crate alloc;

const NET_BUF_LEN: usize = 1526;

/// The VirtIO network device driver.
///
/// `QS` is the VirtIO queue size.
pub struct VirtIoNetDev<H: Hal, T: Transport, const QS: usize> {
    rx_buffers: [Option<NetBufBox>; QS],
    tx_buffers: [Option<NetBufBox>; QS],
    free_tx_bufs: Vec<NetBufBox>,
    buf_pool: Arc<NetBufPool>,
    inner: InnerDev<H, T, QS>,
}

unsafe impl<H: Hal, T: Transport, const QS: usize> Send for VirtIoNetDev<H, T, QS> {}
unsafe impl<H: Hal, T: Transport, const QS: usize> Sync for VirtIoNetDev<H, T, QS> {}

impl<H: Hal, T: Transport, const QS: usize> VirtIoNetDev<H, T, QS> {
    /// Creates a new driver instance and initializes the device, or returns
    /// an error if any step fails.
    pub fn try_new(transport: T) -> DevResult<Self> {
        // 0. Create a new driver instance.
        const NONE_BUF: Option<NetBufBox> = None;
        let inner = InnerDev::new(transport).map_err(as_dev_err)?;
        let rx_buffers = [NONE_BUF; QS];
        let tx_buffers = [NONE_BUF; QS];
        let buf_pool = NetBufPool::new(2 * QS, NET_BUF_LEN)?;
        let free_tx_bufs = Vec::with_capacity(QS);

        let mut dev = Self {
            rx_buffers,
            inner,
            tx_buffers,
            free_tx_bufs,
            buf_pool,
        };

        // 1. Fill all rx buffers.
        for (i, rx_buf_place) in dev.rx_buffers.iter_mut().enumerate() {
            let mut rx_buf = dev.buf_pool.alloc_boxed().ok_or(DevError::NoMemory)?;
            // Safe because the buffer lives as long as the queue.
            let token = unsafe {
                dev.inner
                    .receive_begin(rx_buf.raw_buf_mut())
                    .map_err(as_dev_err)?
            };
            assert_eq!(token, i as u16);
            *rx_buf_place = Some(rx_buf);
        }

        // 2. Allocate all tx buffers.
        for _ in 0..QS {
            let mut tx_buf = dev.buf_pool.alloc_boxed().ok_or(DevError::NoMemory)?;
            // Fill header
            let hdr_len = dev
                .inner
                .fill_buffer_header(tx_buf.raw_buf_mut())
                .or(Err(DevError::InvalidParam))?;
            tx_buf.set_header_len(hdr_len);
            dev.free_tx_bufs.push(tx_buf);
        }

        // 3. Return the driver instance.
        Ok(dev)
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
    #[inline]
    fn mac_address(&self) -> EthernetAddress {
        EthernetAddress(self.inner.mac_address())
    }

    #[inline]
    fn can_transmit(&self) -> bool {
        !self.free_tx_bufs.is_empty() && self.inner.can_transmit()
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

    fn recycle_rx_buffer(&mut self, rx_buf: NetBufPtr) -> DevResult {
        let mut rx_buf = unsafe { NetBuf::from_buf_ptr(rx_buf) };
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

    fn recycle_tx_buffers(&mut self) -> DevResult {
        while let Some(token) = self.inner.poll_transmit() {
            let tx_buf = self.tx_buffers[token as usize]
                .take()
                .ok_or(DevError::BadState)?;
            unsafe {
                self.inner
                    .transmit_complete(token, tx_buf.packet_with_header())
                    .map_err(as_dev_err)?;
            }
            // Recycle the buffer.
            self.free_tx_bufs.push(tx_buf);
        }
        Ok(())
    }

    fn transmit(&mut self, tx_buf: NetBufPtr) -> DevResult {
        // 0. prepare tx buffer.
        let tx_buf = unsafe { NetBuf::from_buf_ptr(tx_buf) };
        // 1. transmit packet.
        let token = unsafe {
            self.inner
                .transmit_begin(tx_buf.packet_with_header())
                .map_err(as_dev_err)?
        };
        self.tx_buffers[token as usize] = Some(tx_buf);
        Ok(())
    }

    fn receive(&mut self) -> DevResult<NetBufPtr> {
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

            Ok(rx_buf.into_buf_ptr())
        } else {
            Err(DevError::Again)
        }
    }

    fn alloc_tx_buffer(&mut self, size: usize) -> DevResult<NetBufPtr> {
        // 0. Allocate a buffer from the queue.
        let mut net_buf = self.free_tx_bufs.pop().ok_or(DevError::NoMemory)?;
        let pkt_len = size;

        // 1. Check if the buffer is large enough.
        let hdr_len = net_buf.header_len();
        if hdr_len + pkt_len > net_buf.capacity() {
            return Err(DevError::InvalidParam);
        }
        net_buf.set_packet_len(pkt_len);

        // 2. Return the buffer.
        Ok(net_buf.into_buf_ptr())
    }
}
