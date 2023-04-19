//! Common traits and types for network device (NIC) drivers.

#![no_std]

#[doc(no_inline)]
pub use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};

/// The ethernet address of the NIC (MAC address).
pub struct EthernetAddress(pub [u8; 6]);

/// The abstract buffer for transmitting or receiving data.
pub trait NetBuffer {
    /// The length of the packet.
    fn packet_len(&self) -> usize;
    /// The reference to the packet data.
    fn packet(&self) -> &[u8];
    /// The mutable reference to the packet data.
    fn packet_mut(&mut self) -> &mut [u8];
}

/// Operations that require a network device (NIC) driver to implement.
pub trait NetDriverOps: BaseDriverOps {
    /// The type of the receive buffer.
    type RxBuffer: NetBuffer;
    /// The type of the transmit buffer.
    type TxBuffer: NetBuffer;

    /// The ethernet address of the NIC.
    fn mac_address(&self) -> EthernetAddress;

    /// Whether can send data.
    fn can_send(&self) -> bool;

    /// Whether can receive data.
    fn can_recv(&self) -> bool;

    /// Allocates a new buffer for transmitting.
    fn new_tx_buffer(&mut self, buf_len: usize) -> DevResult<Self::TxBuffer>;

    /// Gives back the ownership of `rx_buf`, and recycles it for later receiving.
    fn recycle_rx_buffer(&mut self, rx_buf: Self::RxBuffer) -> DevResult;

    /// Sends data in the buffer to the network, and blocks until the request
    /// completed.
    fn send(&mut self, tx_buf: Self::TxBuffer) -> DevResult;

    /// Receives data from the network and store it in the [`RxBuffer`][1],
    /// returns the buffer.
    ///
    /// If currently no data, returns an error with type [`DevError::Again`][2].
    ///
    /// [1]: Self::RxBuffer
    /// [2]: driver_common::DevError::Again
    fn receive(&mut self) -> DevResult<Self::RxBuffer>;
}
