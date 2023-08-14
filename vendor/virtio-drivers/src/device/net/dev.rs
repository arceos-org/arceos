use super::net_buf::NetBuffer;
use super::{EthernetAddress, VirtIONetRaw, NET_HDR_SIZE};
use crate::{hal::Hal, transport::Transport, Error, Result};

/// Driver for a VirtIO block device.
///
/// Unlike [`VirtIONetRaw`], it uses [`NetBuffer`]s for transmission and
/// reception rather than the raw slices. On initialization, it pre-allocates
/// all receive buffers and puts them all in the receive queue.
pub struct VirtIONet<H: Hal, T: Transport, const QUEUE_SIZE: usize> {
    inner: VirtIONetRaw<H, T, QUEUE_SIZE>,
    rx_buffers: [Option<NetBuffer>; QUEUE_SIZE],
}

impl<H: Hal, T: Transport, const QUEUE_SIZE: usize> VirtIONet<H, T, QUEUE_SIZE> {
    /// Create a new VirtIO-Net driver.
    pub fn new(transport: T, buf_len: usize) -> Result<Self> {
        let mut inner = VirtIONetRaw::new(transport)?;

        const NONE_BUF: Option<NetBuffer> = None;
        let mut rx_buffers = [NONE_BUF; QUEUE_SIZE];
        for (i, rx_buf_place) in rx_buffers.iter_mut().enumerate() {
            let mut rx_buf = NetBuffer::new(buf_len);
            // Safe because the buffer lives as long as the queue.
            let token = unsafe { inner.receive_begin(rx_buf.as_bytes_mut())? };
            assert_eq!(token, i as u16);
            *rx_buf_place = Some(rx_buf);
        }

        Ok(VirtIONet { inner, rx_buffers })
    }

    /// Acknowledge interrupt.
    pub fn ack_interrupt(&mut self) -> bool {
        self.inner.ack_interrupt()
    }

    /// Get MAC address.
    pub fn mac_address(&self) -> EthernetAddress {
        self.inner.mac_address()
    }

    /// Whether can transmit packet.
    pub fn can_transmit(&self) -> bool {
        self.inner.can_transmit()
    }

    /// Whether can receive packet.
    pub fn can_receive(&self) -> bool {
        self.inner.can_receive()
    }

    /// Receives a [`NetBuffer`] from network. If currently no data, returns an
    /// error with type [`Error::NotReady`].
    ///
    /// It will try to pop a buffer that completed data reception in the
    /// NIC queue.
    pub fn receive(&mut self) -> Result<NetBuffer> {
        if let Some(token) = self.inner.poll_receive() {
            let mut rx_buf = self.rx_buffers[token as usize]
                .take()
                .ok_or(Error::WrongToken)?;

            // Safe because `token` == `rx_buf.idx`, we are passing the same
            // buffer as we passed to `VirtQueue::add` and it is still valid.
            let (_hdr_len, pkt_len) =
                unsafe { self.inner.receive_complete(token, rx_buf.as_bytes_mut())? };
            rx_buf.set_packet_len(pkt_len);
            Ok(rx_buf)
        } else {
            Err(Error::NotReady)
        }
    }

    /// Gives back the ownership of `rx_buf`, and recycles it for next use.
    ///
    /// It will add the buffer back to the NIC queue.
    pub fn recycle_rx_buffer(&mut self, mut rx_buf: NetBuffer) -> Result {
        // Safe because we take the ownership of `rx_buf` back to `rx_buffers`,
        // it lives as long as the queue.
        let new_token = unsafe { self.inner.receive_begin(rx_buf.as_bytes_mut()) }?;
        // `rx_buffers[new_token]` is expected to be `None` since it was taken
        // away at `Self::receive()` and has not been added back.
        if self.rx_buffers[new_token as usize].is_some() {
            return Err(Error::WrongToken);
        }
        self.rx_buffers[new_token as usize] = Some(rx_buf);
        Ok(())
    }

    /// Allocate a new buffer for transmitting.
    pub fn new_tx_buffer(&self, packet_len: usize) -> Result<NetBuffer> {
        let mut tx_buf = NetBuffer::new(NET_HDR_SIZE + packet_len);
        self.inner.fill_buffer_header(tx_buf.as_bytes_mut())?;
        tx_buf.set_packet_len(packet_len);
        Ok(tx_buf)
    }

    /// Sends a [`NetBuffer`] to the network, and blocks until the request
    /// completed. Returns number of bytes transmitted.
    pub fn transmit_wait(&mut self, tx_buf: NetBuffer) -> Result<usize> {
        self.inner.transmit_wait(tx_buf.as_bytes())
    }
}
