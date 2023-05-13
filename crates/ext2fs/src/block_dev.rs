use core::any::Any;
/// Trait for block devices
/// which reads and writes data in the unit of blocks
pub trait BlockDevice: Send + Sync + Any {
    /// Read data form block to buffer
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    /// Write data from buffer to block
    fn write_block(&self, block_id: usize, buf: &[u8]);
    /// Get block size
    fn block_size(&self) -> usize;
    /// Get block num
    fn block_num(&self) -> usize;
}

pub struct NullDevice;

impl BlockDevice for NullDevice {
    fn read_block(&self, _block_id: usize, _buf: &mut [u8]) {
        panic!("Unimplemented");
    }
    fn write_block(&self, _block_id: usize, _buf: &[u8]) {
        panic!("Unimplemented");
    }
    fn block_num(&self) -> usize {
        panic!("Unimplemented");
    }
    fn block_size(&self) -> usize {
        panic!("Unimplemented");
    }
}