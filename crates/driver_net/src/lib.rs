#![no_std]

use driver_common::{BaseDriverOps, DevResult};

pub struct EthernetAddress(pub [u8; 6]);

pub trait NetBuffer {
    fn packet_len(&self) -> usize;
    fn packet(&self) -> &[u8];
    fn packet_mut(&mut self) -> &mut [u8];
}

pub trait NetDriverOps: BaseDriverOps {
    type RxBuffer: NetBuffer;
    type TxBuffer: NetBuffer;

    fn mac_address(&self) -> EthernetAddress;

    fn can_send(&self) -> bool;
    fn can_recv(&self) -> bool;

    fn new_tx_buffer(&self, buf_len: usize) -> DevResult<Self::TxBuffer>;
    fn recycle_rx_buffer(&self, rx_buf: Self::RxBuffer) -> DevResult;

    fn send(&self, tx_buf: Self::TxBuffer) -> DevResult;
    fn receive(&self) -> DevResult<Self::RxBuffer>;
}
