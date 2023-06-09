use super::{VirtioNetHdr, NET_HDR_SIZE};
use alloc::{vec, vec::Vec};
use zerocopy::AsBytes;

/// A buffer used for receiving and transmitting.
pub struct NetBuffer {
    buf: Vec<usize>, // for alignment
    packet_len: usize,
}

impl NetBuffer {
    /// Allocates a new buffer with length `buf_len`.
    pub(crate) fn new(buf_len: usize) -> Self {
        Self {
            buf: vec![0; (buf_len - 1) / core::mem::size_of::<usize>() + 1],
            packet_len: 0,
        }
    }

    /// Set the network packet length.
    pub(crate) fn set_packet_len(&mut self, packet_len: usize) {
        self.packet_len = packet_len
    }

    /// Returns the network packet length (witout header).
    pub const fn packet_len(&self) -> usize {
        self.packet_len
    }

    /// Returns all data in the buffer, including both the header and the packet.
    pub fn as_bytes(&self) -> &[u8] {
        self.buf.as_bytes()
    }

    /// Returns all data in the buffer with the mutable reference,
    /// including both the header and the packet.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.buf.as_bytes_mut()
    }

    /// Returns the reference of the header.
    pub fn header(&self) -> &VirtioNetHdr {
        unsafe { &*(self.buf.as_ptr() as *const VirtioNetHdr) }
    }

    /// Returns the network packet as a slice.
    pub fn packet(&self) -> &[u8] {
        &self.buf.as_bytes()[NET_HDR_SIZE..NET_HDR_SIZE + self.packet_len]
    }

    /// Returns the network packet as a mutable slice.
    pub fn packet_mut(&mut self) -> &mut [u8] {
        &mut self.buf.as_bytes_mut()[NET_HDR_SIZE..NET_HDR_SIZE + self.packet_len]
    }
}
