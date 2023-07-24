//! Common traits and types for network device (NIC) drivers.

#![no_std]
#![feature(const_mut_refs)]
#![feature(const_slice_from_raw_parts_mut)]
#![feature(box_into_inner)]

#[cfg(feature = "ixgbe")]
/// ixgbe NIC device driver.
pub mod ixgbe;
mod net_buf;

use core::ptr::NonNull;

#[doc(no_inline)]
pub use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};

pub use self::net_buf::{NetBuf, NetBufBox, NetBufPool};

/// The ethernet address of the NIC (MAC address).
pub struct EthernetAddress(pub [u8; 6]);

/// Operations that require a network device (NIC) driver to implement.
pub trait NetDriverOps: BaseDriverOps {
    /// The ethernet address of the NIC.
    fn mac_address(&self) -> EthernetAddress;

    /// Whether can transmit packets.
    fn can_transmit(&self) -> bool;

    /// Whether can receive packets.
    fn can_receive(&self) -> bool;

    /// Size of the receive queue.
    fn rx_queue_size(&self) -> usize;

    /// Size of the transmit queue.
    fn tx_queue_size(&self) -> usize;

    /// Gives back the `rx_buf` to the receive queue for later receiving.
    ///
    /// `rx_buf` should be the same as the one returned by
    /// [`NetDriverOps::receive`].
    fn recycle_rx_buffer(&mut self, rx_buf: NetBufPtr) -> DevResult;

    /// Poll the transmit queue and gives back the buffers for previous transmiting.
    /// returns [`DevResult`].
    fn recycle_tx_buffers(&mut self) -> DevResult;

    /// Transmits a packet in the buffer to the network, without blocking,
    /// returns [`DevResult`].
    fn transmit(&mut self, tx_buf: NetBufPtr) -> DevResult;

    /// Receives a packet from the network and store it in the [`NetBuf`],
    /// returns the buffer.
    ///
    /// Before receiving, the driver should have already populated some buffers
    /// in the receive queue by [`NetDriverOps::recycle_rx_buffer`].
    ///
    /// If currently no incomming packets, returns an error with type
    /// [`DevError::Again`].
    fn receive(&mut self) -> DevResult<NetBufPtr>;

    /// Allocate a memory buffer of a specified size for network transmission,
    /// returns [`DevResult`]
    fn alloc_tx_buffer(&mut self, size: usize) -> DevResult<NetBufPtr>;
}

/// A raw buffer struct for network device.
pub struct NetBufPtr {
    // The raw pointer of the original object.
    raw_ptr: NonNull<u8>,
    // The pointer to the net buffer.
    buf_ptr: NonNull<u8>,
    len: usize,
}

impl NetBufPtr {
    /// Create a new [`NetBufPtr`].
    pub fn new(raw_ptr: NonNull<u8>, buf_ptr: NonNull<u8>, len: usize) -> Self {
        Self {
            raw_ptr,
            buf_ptr,
            len,
        }
    }

    /// Return raw pointer of the original object.
    pub fn raw_ptr<T>(&self) -> *mut T {
        self.raw_ptr.as_ptr() as *mut T
    }

    /// Return [`NetBufPtr`] buffer len.
    pub fn packet_len(&self) -> usize {
        self.len
    }

    /// Return [`NetBufPtr`] buffer as &[u8].
    pub fn packet(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.buf_ptr.as_ptr() as *const u8, self.len) }
    }

    /// Return [`NetBufPtr`] buffer as &mut [u8].
    pub fn packet_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.buf_ptr.as_ptr(), self.len) }
    }
}
