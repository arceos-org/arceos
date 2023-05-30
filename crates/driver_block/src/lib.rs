//! Common traits and types for block storage device drivers (i.e. disk).

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(const_trait_impl)]

#[cfg(feature = "ramdisk")]
pub mod ramdisk;

#[doc(no_inline)]
pub use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};

use virtio_drivers::device::blk::{BlkReq, BlkResp};

/// Operations that require a block storage device driver to implement.
pub trait BlockDriverOps: BaseDriverOps {
    /// The number of blocks in this storage device.
    ///
    /// The total size of the device is `num_blocks() * block_size()`.
    fn num_blocks(&self) -> u64;
    /// The size of each block in bytes.
    fn block_size(&self) -> usize;

    /// Reads blocked data from the given block.
    ///
    /// The size of the buffer may exceed the block size, in which case multiple
    /// contiguous blocks will be read.
    fn read_block(&mut self, block_id: u64, buf: &mut [u8]) -> DevResult;

    /// Writes blocked data to the given block.
    ///
    /// The size of the buffer may exceed the block size, in which case multiple
    /// contiguous blocks will be written.
    fn write_block(&mut self, block_id: u64, buf: &[u8]) -> DevResult;

    /// Flushes the device to write all pending data to the storage.
    fn flush(&mut self) -> DevResult;

    #[cfg(feature = "irq")]
    /// Reads blocked data from the given block, without blocking.
    fn read_block_nb(
        &mut self,
        block_id: u64,
        req: &mut BlkReq,
        buf: &mut [u8],
        resp: &mut BlkResp,
    ) -> DevResult<u16>;

    /// Writes blocked data to the given block, without blocking.
    #[cfg(feature = "irq")]
    fn write_block_nb(
        &mut self,
        block_id: u64,
        req: &mut BlkReq,
        buf: &[u8],
        resp: &mut BlkResp,
    ) -> DevResult<u16>;

    /// Complete a read block request.
    #[cfg(feature = "irq")]
    fn complete_read_block(
        &mut self,
        token: u16,
        req: &BlkReq,
        buf: &mut [u8],
        resp: &mut BlkResp,
    ) -> DevResult;

    /// Complete a write block request.
    #[cfg(feature = "irq")]
    fn complete_write_block(
        &mut self,
        token: u16,
        req: &BlkReq,
        buf: &[u8],
        resp: &mut BlkResp,
    ) -> DevResult;

    /// Peek the used ring.
    #[cfg(feature = "irq")]
    fn peek_used(&mut self) -> Option<u16>;
}
