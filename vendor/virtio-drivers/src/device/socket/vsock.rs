//! Driver for VirtIO socket devices.
#![deny(unsafe_op_in_unsafe_fn)]

use super::error::SocketError;
use super::protocol::{VirtioVsockConfig, VirtioVsockHdr, VirtioVsockOp, VsockAddr};
use crate::device::common::Feature;
use crate::hal::{BufferDirection, Dma, Hal};
use crate::queue::VirtQueue;
use crate::transport::Transport;
use crate::volatile::volread;
use crate::Result;
use core::hint::spin_loop;
use core::mem::size_of;
use core::ptr::NonNull;
use log::{debug, info};
use zerocopy::{AsBytes, FromBytes};

const RX_QUEUE_IDX: u16 = 0;
const TX_QUEUE_IDX: u16 = 1;
const EVENT_QUEUE_IDX: u16 = 2;

const QUEUE_SIZE: usize = 8;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ConnectionInfo {
    dst: VsockAddr,
    src_port: u32,
    /// The last `buf_alloc` value the peer sent to us, indicating how much receive buffer space in
    /// bytes it has allocated for packet bodies.
    peer_buf_alloc: u32,
    /// The last `fwd_cnt` value the peer sent to us, indicating how many bytes of packet bodies it
    /// has finished processing.
    peer_fwd_cnt: u32,
    /// The number of bytes of packet bodies which we have sent to the peer.
    tx_cnt: u32,
    /// The number of bytes of packet bodies which we have received from the peer and handled.
    fwd_cnt: u32,
    /// Whether we have recently requested credit from the peer.
    ///
    /// This is set to true when we send a `VIRTIO_VSOCK_OP_CREDIT_REQUEST`, and false when we
    /// receive a `VIRTIO_VSOCK_OP_CREDIT_UPDATE`.
    has_pending_credit_request: bool,
}

impl ConnectionInfo {
    fn peer_free(&self) -> u32 {
        self.peer_buf_alloc - (self.tx_cnt - self.peer_fwd_cnt)
    }

    fn new_header(&self, src_cid: u64) -> VirtioVsockHdr {
        VirtioVsockHdr {
            src_cid: src_cid.into(),
            dst_cid: self.dst.cid.into(),
            src_port: self.src_port.into(),
            dst_port: self.dst.port.into(),
            fwd_cnt: self.fwd_cnt.into(),
            ..Default::default()
        }
    }
}

/// An event received from a VirtIO socket device.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VsockEvent {
    /// The source of the event, i.e. the peer who sent it.
    pub source: VsockAddr,
    /// The destination of the event, i.e. the CID and port on our side.
    pub destination: VsockAddr,
    /// The type of event.
    pub event_type: VsockEventType,
}

/// The reason why a vsock connection was closed.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DisconnectReason {
    /// The peer has either closed the connection in response to our shutdown request, or forcibly
    /// closed it of its own accord.
    Reset,
    /// The peer asked to shut down the connection.
    Shutdown,
}

/// Details of the type of an event received from a VirtIO socket.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VsockEventType {
    /// The connection was successfully established.
    Connected,
    /// The connection was closed.
    Disconnected {
        /// The reason for the disconnection.
        reason: DisconnectReason,
    },
    /// Data was received on the connection.
    Received {
        /// The length of the data in bytes.
        length: usize,
    },
}

/// Driver for a VirtIO socket device.
pub struct VirtIOSocket<H: Hal, T: Transport> {
    transport: T,
    /// Virtqueue to receive packets.
    rx: VirtQueue<H, { QUEUE_SIZE }>,
    tx: VirtQueue<H, { QUEUE_SIZE }>,
    /// Virtqueue to receive events from the device.
    event: VirtQueue<H, { QUEUE_SIZE }>,
    /// The guest_cid field contains the guest’s context ID, which uniquely identifies
    /// the device for its lifetime. The upper 32 bits of the CID are reserved and zeroed.
    guest_cid: u64,
    rx_buf_dma: Dma<H>,

    /// Currently the device is only allowed to be connected to one destination at a time.
    connection_info: Option<ConnectionInfo>,
}

impl<H: Hal, T: Transport> Drop for VirtIOSocket<H, T> {
    fn drop(&mut self) {
        // Clear any pointers pointing to DMA regions, so the device doesn't try to access them
        // after they have been freed.
        self.transport.queue_unset(RX_QUEUE_IDX);
        self.transport.queue_unset(TX_QUEUE_IDX);
        self.transport.queue_unset(EVENT_QUEUE_IDX);
    }
}

impl<H: Hal, T: Transport> VirtIOSocket<H, T> {
    /// Create a new VirtIO Vsock driver.
    pub fn new(mut transport: T) -> Result<Self> {
        transport.begin_init(|features| {
            let features = Feature::from_bits_truncate(features);
            info!("Device features: {:?}", features);
            // negotiate these flags only
            let supported_features = Feature::empty();
            (features & supported_features).bits()
        });

        let config = transport.config_space::<VirtioVsockConfig>()?;
        info!("config: {:?}", config);
        // Safe because config is a valid pointer to the device configuration space.
        let guest_cid = unsafe {
            volread!(config, guest_cid_low) as u64 | (volread!(config, guest_cid_high) as u64) << 32
        };
        info!("guest cid: {guest_cid:?}");

        let mut rx = VirtQueue::new(&mut transport, RX_QUEUE_IDX)?;
        let tx = VirtQueue::new(&mut transport, TX_QUEUE_IDX)?;
        let event = VirtQueue::new(&mut transport, EVENT_QUEUE_IDX)?;

        // Allocates 4 KiB memory as the rx buffer.
        let rx_buf_dma = Dma::new(
            1, // pages
            BufferDirection::DeviceToDriver,
        )?;
        let rx_buf = rx_buf_dma.raw_slice();
        // Safe because `rx_buf` lives as long as the `rx` queue.
        unsafe {
            Self::fill_rx_queue(&mut rx, rx_buf, &mut transport)?;
        }
        transport.finish_init();

        Ok(Self {
            transport,
            rx,
            tx,
            event,
            guest_cid,
            rx_buf_dma,
            connection_info: None,
        })
    }

    /// Fills the `rx` queue with the buffer `rx_buf`.
    ///
    /// # Safety
    ///
    /// `rx_buf` must live at least as long as the `rx` queue, and the parts of the buffer which are
    /// in the queue must not be used anywhere else at the same time.
    unsafe fn fill_rx_queue(
        rx: &mut VirtQueue<H, { QUEUE_SIZE }>,
        rx_buf: NonNull<[u8]>,
        transport: &mut T,
    ) -> Result {
        if rx_buf.len() < size_of::<VirtioVsockHdr>() * QUEUE_SIZE {
            return Err(SocketError::BufferTooShort.into());
        }
        for i in 0..QUEUE_SIZE {
            // Safe because the buffer lives as long as the queue, as specified in the function
            // safety requirement, and we don't access it until it is popped.
            unsafe {
                let buffer = Self::as_mut_sub_rx_buffer(rx_buf, i)?;
                let token = rx.add(&[], &mut [buffer])?;
                assert_eq!(i, token.into());
            }
        }

        if rx.should_notify() {
            transport.notify(RX_QUEUE_IDX);
        }
        Ok(())
    }

    /// Sends a request to connect to the given destination.
    ///
    /// This returns as soon as the request is sent; you should wait until `poll_recv` returns a
    /// `VsockEventType::Connected` event indicating that the peer has accepted the connection
    /// before sending data.
    pub fn connect(&mut self, dst_cid: u64, src_port: u32, dst_port: u32) -> Result {
        if self.connection_info.is_some() {
            return Err(SocketError::ConnectionExists.into());
        }
        let new_connection_info = ConnectionInfo {
            dst: VsockAddr {
                cid: dst_cid,
                port: dst_port,
            },
            src_port,
            ..Default::default()
        };
        let header = VirtioVsockHdr {
            op: VirtioVsockOp::Request.into(),
            ..new_connection_info.new_header(self.guest_cid)
        };
        // Sends a header only packet to the tx queue to connect the device to the listening
        // socket at the given destination.
        self.send_packet_to_tx_queue(&header, &[])?;

        self.connection_info = Some(new_connection_info);
        debug!("Connection requested: {:?}", self.connection_info);
        Ok(())
    }

    /// Blocks until the peer either accepts our connection request (with a
    /// `VIRTIO_VSOCK_OP_RESPONSE`) or rejects it (with a
    /// `VIRTIO_VSOCK_OP_RST`).
    pub fn wait_for_connect(&mut self) -> Result {
        match self.wait_for_recv(&mut [])?.event_type {
            VsockEventType::Connected => Ok(()),
            VsockEventType::Disconnected { .. } => Err(SocketError::ConnectionFailed.into()),
            VsockEventType::Received { .. } => Err(SocketError::InvalidOperation.into()),
        }
    }

    /// Requests the peer to send us a credit update for the current connection.
    fn request_credit(&mut self) -> Result {
        let connection_info = self.connection_info()?;
        let header = VirtioVsockHdr {
            op: VirtioVsockOp::CreditRequest.into(),
            ..connection_info.new_header(self.guest_cid)
        };
        self.send_packet_to_tx_queue(&header, &[])
    }

    /// Sends the buffer to the destination.
    pub fn send(&mut self, buffer: &[u8]) -> Result {
        let mut connection_info = self.connection_info()?;

        let result = self.check_peer_buffer_is_sufficient(&mut connection_info, buffer.len());
        self.connection_info = Some(connection_info.clone());
        result?;

        let len = buffer.len() as u32;
        let header = VirtioVsockHdr {
            op: VirtioVsockOp::Rw.into(),
            len: len.into(),
            buf_alloc: 0.into(),
            ..connection_info.new_header(self.guest_cid)
        };
        self.connection_info.as_mut().unwrap().tx_cnt += len;
        self.send_packet_to_tx_queue(&header, buffer)
    }

    fn check_peer_buffer_is_sufficient(
        &mut self,
        connection_info: &mut ConnectionInfo,
        buffer_len: usize,
    ) -> Result {
        if connection_info.peer_free() as usize >= buffer_len {
            Ok(())
        } else {
            // Request an update of the cached peer credit, if we haven't already done so, and tell
            // the caller to try again later.
            if !connection_info.has_pending_credit_request {
                self.request_credit()?;
                connection_info.has_pending_credit_request = true;
            }
            Err(SocketError::InsufficientBufferSpaceInPeer.into())
        }
    }

    /// Polls the vsock device to receive data or other updates.
    ///
    /// A buffer must be provided to put the data in if there is some to
    /// receive.
    pub fn poll_recv(&mut self, buffer: &mut [u8]) -> Result<Option<VsockEvent>> {
        let connection_info = self.connection_info()?;

        // Tell the peer that we have space to receive some data.
        let header = VirtioVsockHdr {
            op: VirtioVsockOp::CreditUpdate.into(),
            buf_alloc: (buffer.len() as u32).into(),
            ..connection_info.new_header(self.guest_cid)
        };
        self.send_packet_to_tx_queue(&header, &[])?;

        // Handle entries from the RX virtqueue until we find one that generates an event.
        let event = self.poll_rx_queue(buffer)?;

        if self.rx.should_notify() {
            self.transport.notify(RX_QUEUE_IDX);
        }

        Ok(event)
    }

    /// Blocks until we get some event from the vsock device.
    ///
    /// A buffer must be provided to put the data in if there is some to
    /// receive.
    pub fn wait_for_recv(&mut self, buffer: &mut [u8]) -> Result<VsockEvent> {
        loop {
            if let Some(event) = self.poll_recv(buffer)? {
                return Ok(event);
            } else {
                spin_loop();
            }
        }
    }

    /// Request to shut down the connection cleanly.
    ///
    /// This returns as soon as the request is sent; you should wait until `poll_recv` returns a
    /// `VsockEventType::Disconnected` event if you want to know that the peer has acknowledged the
    /// shutdown.
    pub fn shutdown(&mut self) -> Result {
        let connection_info = self.connection_info()?;
        let header = VirtioVsockHdr {
            op: VirtioVsockOp::Shutdown.into(),
            ..connection_info.new_header(self.guest_cid)
        };
        self.send_packet_to_tx_queue(&header, &[])
    }

    /// Forcibly closes the connection without waiting for the peer.
    pub fn force_close(&mut self) -> Result {
        let connection_info = self.connection_info()?;
        let header = VirtioVsockHdr {
            op: VirtioVsockOp::Rst.into(),
            ..connection_info.new_header(self.guest_cid)
        };
        self.send_packet_to_tx_queue(&header, &[])?;
        self.connection_info = None;
        Ok(())
    }

    fn send_packet_to_tx_queue(&mut self, header: &VirtioVsockHdr, buffer: &[u8]) -> Result {
        let _len = self.tx.add_notify_wait_pop(
            &[header.as_bytes(), buffer],
            &mut [],
            &mut self.transport,
        )?;
        Ok(())
    }

    /// Polls the RX virtqueue until either it is empty, there is an error, or we find a packet
    /// which generates a `VsockEvent`.
    ///
    /// Returns `Ok(None)` if the virtqueue is empty, possibly after processing some packets which
    /// don't result in any events to return.
    fn poll_rx_queue(&mut self, body: &mut [u8]) -> Result<Option<VsockEvent>> {
        loop {
            let mut connection_info = self.connection_info.clone().unwrap_or_default();
            let Some(header) = self.pop_packet_from_rx_queue(body)? else{
                return Ok(None);
            };

            let op = header.op()?;

            // Skip packets which don't match our current connection.
            if header.source() != connection_info.dst
                || header.dst_cid.get() != self.guest_cid
                || header.dst_port.get() != connection_info.src_port
            {
                debug!(
                    "Skipping {:?} as connection is {:?}",
                    header, connection_info
                );
                continue;
            }

            connection_info.peer_buf_alloc = header.buf_alloc.into();
            connection_info.peer_fwd_cnt = header.fwd_cnt.into();
            if self.connection_info.is_some() {
                self.connection_info = Some(connection_info.clone());
                debug!("Connection info updated: {:?}", self.connection_info);
            }

            match op {
                VirtioVsockOp::Request => {
                    header.check_data_is_empty()?;
                    // TODO: Send a Rst, or support listening.
                }
                VirtioVsockOp::Response => {
                    header.check_data_is_empty()?;
                    return Ok(Some(VsockEvent {
                        source: connection_info.dst,
                        destination: VsockAddr {
                            cid: self.guest_cid,
                            port: connection_info.src_port,
                        },
                        event_type: VsockEventType::Connected,
                    }));
                }
                VirtioVsockOp::CreditUpdate => {
                    header.check_data_is_empty()?;
                    connection_info.has_pending_credit_request = false;
                    if self.connection_info.is_some() {
                        self.connection_info = Some(connection_info.clone());
                    }

                    // Virtio v1.1 5.10.6.3
                    // The driver can also receive a VIRTIO_VSOCK_OP_CREDIT_UPDATE packet without previously
                    // sending a VIRTIO_VSOCK_OP_CREDIT_REQUEST packet. This allows communicating updates
                    // any time a change in buffer space occurs.
                    continue;
                }
                VirtioVsockOp::Rst | VirtioVsockOp::Shutdown => {
                    header.check_data_is_empty()?;

                    self.connection_info = None;
                    info!("Disconnected from the peer");

                    let reason = if op == VirtioVsockOp::Rst {
                        DisconnectReason::Reset
                    } else {
                        DisconnectReason::Shutdown
                    };
                    return Ok(Some(VsockEvent {
                        source: connection_info.dst,
                        destination: VsockAddr {
                            cid: self.guest_cid,
                            port: connection_info.src_port,
                        },
                        event_type: VsockEventType::Disconnected { reason },
                    }));
                }
                VirtioVsockOp::Rw => {
                    self.connection_info.as_mut().unwrap().fwd_cnt += header.len();
                    return Ok(Some(VsockEvent {
                        source: connection_info.dst,
                        destination: VsockAddr {
                            cid: self.guest_cid,
                            port: connection_info.src_port,
                        },
                        event_type: VsockEventType::Received {
                            length: header.len() as usize,
                        },
                    }));
                }
                VirtioVsockOp::CreditRequest => {
                    header.check_data_is_empty()?;
                    // TODO: Send a credit update.
                }
                VirtioVsockOp::Invalid => {
                    return Err(SocketError::InvalidOperation.into());
                }
            }
        }
    }

    /// Pops one packet from the RX queue, if there is one pending. Returns the header, and copies
    /// the body into the given buffer.
    ///
    /// Returns `None` if there is no pending packet, or an error if the body is bigger than the
    /// buffer supplied.
    fn pop_packet_from_rx_queue(&mut self, body: &mut [u8]) -> Result<Option<VirtioVsockHdr>> {
        let Some(token) = self.rx.peek_used() else {
            return Ok(None);
        };

        // Safe because we maintain a consistent mapping of tokens to buffers, so we pass the same
        // buffer to `pop_used` as we previously passed to `add` for the token. Once we add the
        // buffer back to the RX queue then we don't access it again until next time it is popped.
        let header = unsafe {
            let buffer = Self::as_mut_sub_rx_buffer(self.rx_buf_dma.raw_slice(), token.into())?;
            let _len = self.rx.pop_used(token, &[], &mut [buffer])?;

            // Read the header and body from the buffer. Don't check the result yet, because we need
            // to add the buffer back to the queue either way.
            let header_result = read_header_and_body(buffer, body);

            // Add the buffer back to the RX queue.
            let new_token = self.rx.add(&[], &mut [buffer])?;
            // If the RX buffer somehow gets assigned a different token, then our safety assumptions
            // are broken and we can't safely continue to do anything with the device.
            assert_eq!(new_token, token);

            header_result
        }?;

        debug!("Received packet {:?}. Op {:?}", header, header.op());
        Ok(Some(header))
    }

    fn connection_info(&self) -> Result<ConnectionInfo> {
        self.connection_info
            .clone()
            .ok_or(SocketError::NotConnected.into())
    }

    /// Gets a reference to a subslice of the RX buffer to be used for the given entry in the RX
    /// virtqueue.
    ///
    /// # Safety
    ///
    /// `rx_buf` must be a valid dereferenceable pointer.
    /// The returned reference has an arbitrary lifetime `'a`. This lifetime must not overlap with
    /// any other references to the same subslice of the RX buffer or outlive the buffer.
    unsafe fn as_mut_sub_rx_buffer<'a>(
        mut rx_buf: NonNull<[u8]>,
        i: usize,
    ) -> Result<&'a mut [u8]> {
        let buffer_size = rx_buf.len() / QUEUE_SIZE;
        let start = buffer_size
            .checked_mul(i)
            .ok_or(SocketError::InvalidNumber)?;
        let end = start
            .checked_add(buffer_size)
            .ok_or(SocketError::InvalidNumber)?;
        // Safe because no alignment or initialisation is required for [u8], and our caller assures
        // us that `rx_buf` is dereferenceable and that the lifetime of the slice we are creating
        // won't overlap with any other references to the same slice or outlive it.
        unsafe {
            rx_buf
                .as_mut()
                .get_mut(start..end)
                .ok_or(SocketError::BufferTooShort.into())
        }
    }
}

fn read_header_and_body(buffer: &[u8], body: &mut [u8]) -> Result<VirtioVsockHdr> {
    let header = VirtioVsockHdr::read_from_prefix(buffer).ok_or(SocketError::BufferTooShort)?;
    let body_length = header.len() as usize;
    let data_end = size_of::<VirtioVsockHdr>()
        .checked_add(body_length)
        .ok_or(SocketError::InvalidNumber)?;
    let data = buffer
        .get(size_of::<VirtioVsockHdr>()..data_end)
        .ok_or(SocketError::BufferTooShort)?;
    body.get_mut(0..body_length)
        .ok_or(SocketError::OutputBufferTooShort(body_length))?
        .copy_from_slice(data);
    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::volatile::ReadOnly;
    use crate::{
        hal::fake::FakeHal,
        transport::{
            fake::{FakeTransport, QueueStatus, State},
            DeviceStatus, DeviceType,
        },
    };
    use alloc::{sync::Arc, vec};
    use core::ptr::NonNull;
    use std::sync::Mutex;

    #[test]
    fn config() {
        let mut config_space = VirtioVsockConfig {
            guest_cid_low: ReadOnly::new(66),
            guest_cid_high: ReadOnly::new(0),
        };
        let state = Arc::new(Mutex::new(State {
            status: DeviceStatus::empty(),
            driver_features: 0,
            guest_page_size: 0,
            interrupt_pending: false,
            queues: vec![QueueStatus::default(); 3],
        }));
        let transport = FakeTransport {
            device_type: DeviceType::Socket,
            max_queue_size: 32,
            device_features: 0,
            config_space: NonNull::from(&mut config_space),
            state: state.clone(),
        };
        let socket =
            VirtIOSocket::<FakeHal, FakeTransport<VirtioVsockConfig>>::new(transport).unwrap();
        assert_eq!(socket.guest_cid, 0x00_0000_0042);
    }
}
