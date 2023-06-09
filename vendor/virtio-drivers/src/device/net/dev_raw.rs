use super::{Config, EthernetAddress, Features, VirtioNetHdr};
use super::{MIN_BUFFER_LEN, NET_HDR_SIZE, QUEUE_RECEIVE, QUEUE_TRANSMIT};
use crate::hal::Hal;
use crate::queue::VirtQueue;
use crate::transport::Transport;
use crate::volatile::volread;
use crate::{Error, Result};
use log::{debug, info, warn};
use zerocopy::AsBytes;

/// Raw driver for a VirtIO block device.
///
/// This is a raw version of the VirtIONet driver. It provides non-blocking
/// methods for transmitting and receiving raw slices, without the buffer
/// management. For more higher-level fucntions such as receive buffer backing,
/// see [`VirtIONet`].
///
/// [`VirtIONet`]: super::VirtIONet
pub struct VirtIONetRaw<H: Hal, T: Transport, const QUEUE_SIZE: usize> {
    transport: T,
    mac: EthernetAddress,
    recv_queue: VirtQueue<H, QUEUE_SIZE>,
    xmit_queue: VirtQueue<H, QUEUE_SIZE>,
}

impl<H: Hal, T: Transport, const QUEUE_SIZE: usize> VirtIONetRaw<H, T, QUEUE_SIZE> {
    /// Create a new VirtIO-Net driver.
    pub fn new(mut transport: T) -> Result<Self> {
        transport.begin_init(|features| {
            let features = Features::from_bits_truncate(features);
            info!("Device features {:?}", features);
            let supported_features = Features::MAC | Features::STATUS;
            (features & supported_features).bits()
        });
        // read configuration space
        let config = transport.config_space::<Config>()?;
        let mac;
        // Safe because config points to a valid MMIO region for the config space.
        unsafe {
            mac = volread!(config, mac);
            debug!(
                "Got MAC={:02x?}, status={:?}",
                mac,
                volread!(config, status)
            );
        }

        let recv_queue = VirtQueue::new(&mut transport, QUEUE_RECEIVE)?;
        let xmit_queue = VirtQueue::new(&mut transport, QUEUE_TRANSMIT)?;
        transport.finish_init();

        Ok(VirtIONetRaw {
            transport,
            mac,
            recv_queue,
            xmit_queue,
        })
    }

    /// Acknowledge interrupt.
    pub fn ack_interrupt(&mut self) -> bool {
        self.transport.ack_interrupt()
    }

    /// Get MAC address.
    pub fn mac_address(&self) -> EthernetAddress {
        self.mac
    }

    /// Whether can transmit packet.
    pub fn can_transmit(&self) -> bool {
        self.xmit_queue.available_desc() >= 1
    }

    /// Whether can receive packet.
    pub fn can_receive(&self) -> bool {
        self.recv_queue.can_pop()
    }

    /// Whether the length of the receive buffer is valid.
    fn check_rx_buf_len(rx_buf: &[u8]) -> Result<()> {
        if rx_buf.len() < MIN_BUFFER_LEN {
            warn!("Receive buffer len {} is too small", rx_buf.len());
            Err(Error::InvalidParam)
        } else {
            Ok(())
        }
    }

    /// Whether the length of the transmit buffer is valid.
    fn check_tx_buf_len(tx_buf: &[u8]) -> Result<()> {
        if tx_buf.len() < NET_HDR_SIZE {
            warn!("Transmit buffer len {} is too small", tx_buf.len());
            Err(Error::InvalidParam)
        } else {
            Ok(())
        }
    }

    /// Fill the header of the `buffer` with [`VirtioNetHdr`].
    ///
    /// If the `buffer` is not large enough, it returns [`Error::InvalidParam`].
    pub fn fill_buffer_header(&self, buffer: &mut [u8]) -> Result<usize> {
        if buffer.len() < NET_HDR_SIZE {
            return Err(Error::InvalidParam);
        }
        let header = VirtioNetHdr::default();
        buffer[..NET_HDR_SIZE].copy_from_slice(header.as_bytes());
        Ok(NET_HDR_SIZE)
    }

    /// Submits a request to transmit a buffer immediately without waiting for
    /// the transmission to complete.
    ///
    /// It will submit request to the VirtIO net device and return a token
    /// identifying the position of the first descriptor in the chain. If there
    /// are not enough descriptors to allocate, then it returns
    /// [`Error::QueueFull`].
    ///
    /// The caller needs to fill the `tx_buf` with a header by calling
    /// [`fill_buffer_header`] before transmission. Then it calls [`poll_transmit`]
    /// with the returned token to check whether the device has finished handling
    /// the request. Once it has, the caller must call [`transmit_complete`] with
    /// the same buffers before reading the result (transmitted length).
    ///
    /// # Safety
    ///
    /// `tx_buf` is still borrowed by the underlying VirtIO net device even after
    /// this method returns. Thus, it is the caller's responsibility to guarantee
    /// that they are not accessed before the request is completed in order to
    /// avoid data races.
    ///
    /// [`fill_buffer_header`]: Self::fill_buffer_header
    /// [`poll_transmit`]: Self::poll_transmit
    /// [`transmit_complete`]: Self::transmit_complete
    pub unsafe fn transmit_begin(&mut self, tx_buf: &[u8]) -> Result<u16> {
        Self::check_tx_buf_len(tx_buf)?;
        let token = self.xmit_queue.add(&[tx_buf], &mut [])?;
        if self.xmit_queue.should_notify() {
            self.transport.notify(QUEUE_TRANSMIT);
        }
        Ok(token)
    }

    /// Fetches the token of the next completed transmission request from the
    /// used ring and returns it, without removing it from the used ring. If
    /// there are no pending completed requests it returns [`None`].
    pub fn poll_transmit(&mut self) -> Option<u16> {
        self.xmit_queue.peek_used()
    }

    /// Completes a transmission operation which was started by [`transmit_begin`].
    /// Returns number of bytes transmitted.
    ///
    /// # Safety
    ///
    /// The same buffers must be passed in again as were passed to
    /// [`transmit_begin`] when it returned the token.
    ///
    /// [`transmit_begin`]: Self::transmit_begin
    pub unsafe fn transmit_complete(&mut self, token: u16, tx_buf: &[u8]) -> Result<usize> {
        let len = self.xmit_queue.pop_used(token, &[tx_buf], &mut [])?;
        Ok(len as usize)
    }

    /// Submits a request to receive a buffer immediately without waiting for
    /// the reception to complete.
    ///
    /// It will submit request to the VirtIO net device and return a token
    /// identifying the position of the first descriptor in the chain. If there
    /// are not enough descriptors to allocate, then it returns
    /// [`Error::QueueFull`].
    ///
    /// The caller can then call [`poll_receive`] with the returned token to
    /// check whether the device has finished handling the request. Once it has,
    /// the caller must call [`receive_complete`] with the same buffers before
    /// reading the response.
    ///
    /// # Safety
    ///
    /// `rx_buf` is still borrowed by the underlying VirtIO net device even after
    /// this method returns. Thus, it is the caller's responsibility to guarantee
    /// that they are not accessed before the request is completed in order to
    /// avoid data races.
    ///
    /// [`poll_receive`]: Self::poll_receive
    /// [`receive_complete`]: Self::receive_complete
    pub unsafe fn receive_begin(&mut self, rx_buf: &mut [u8]) -> Result<u16> {
        Self::check_rx_buf_len(rx_buf)?;
        let token = self.recv_queue.add(&[], &mut [rx_buf])?;
        if self.recv_queue.should_notify() {
            self.transport.notify(QUEUE_RECEIVE);
        }
        Ok(token)
    }

    /// Fetches the token of the next completed reception request from the
    /// used ring and returns it, without removing it from the used ring. If
    /// there are no pending completed requests it returns [`None`].
    pub fn poll_receive(&mut self) -> Option<u16> {
        self.recv_queue.peek_used()
    }

    /// Completes a transmission operation which was started by [`receive_begin`].
    ///
    /// After completion, the `rx_buf` will contain a header followed by the
    /// received packet. It returns the length of the header and the length of
    /// the packet.
    ///
    /// # Safety
    ///
    /// The same buffers must be passed in again as were passed to
    /// [`receive_begin`] when it returned the token.
    ///
    /// [`receive_begin`]: Self::receive_begin
    pub unsafe fn receive_complete(
        &mut self,
        token: u16,
        rx_buf: &mut [u8],
    ) -> Result<(usize, usize)> {
        let len = self.recv_queue.pop_used(token, &[], &mut [rx_buf])? as usize;
        let packet_len = len.checked_sub(NET_HDR_SIZE).ok_or(Error::IoError)?;
        Ok((NET_HDR_SIZE, packet_len))
    }

    /// Transmits a packet to the network, and blocks until the request
    /// completed. Returns number of bytes transmitted.
    ///
    /// The caller needs to fill the `tx_buf` with a header by calling
    /// [`fill_buffer_header`] before transmission.
    ///
    /// [`fill_buffer_header`]: Self::fill_buffer_header
    pub fn transmit_wait(&mut self, tx_buf: &[u8]) -> Result<usize> {
        let token = unsafe { self.transmit_begin(tx_buf)? };
        while self.poll_transmit().is_none() {
            core::hint::spin_loop();
        }
        unsafe { self.transmit_complete(token, tx_buf) }
    }

    /// Blocks and waits for a packet to be received.
    ///
    /// After completion, the `rx_buf` will contain a header followed by the
    /// received packet. It returns the length of the header and the length of
    /// the packet.
    pub fn receive_wait(&mut self, rx_buf: &mut [u8]) -> Result<(usize, usize)> {
        let token = unsafe { self.receive_begin(rx_buf)? };
        while self.poll_receive().is_none() {
            core::hint::spin_loop();
        }
        unsafe { self.receive_complete(token, rx_buf) }
    }
}

impl<H: Hal, T: Transport, const QUEUE_SIZE: usize> Drop for VirtIONetRaw<H, T, QUEUE_SIZE> {
    fn drop(&mut self) {
        // Clear any pointers pointing to DMA regions, so the device doesn't try to access them
        // after they have been freed.
        self.transport.queue_unset(QUEUE_RECEIVE);
        self.transport.queue_unset(QUEUE_TRANSMIT);
    }
}
