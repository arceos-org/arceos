#[cfg(not(feature = "journal"))]
use core::any::Any;
/// Trait for block devices
/// which reads and writes data in the unit of blocks
#[cfg(not(feature = "journal"))]
pub trait BlockDevice: Any {
    ///Read data form block to buffer
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    ///Write data from buffer to block
    fn write_block(&self, block_id: usize, buf: &[u8]);
}
#[cfg(feature = "journal")]
pub trait BlockDevice = jbd::sal::BlockDevice;
