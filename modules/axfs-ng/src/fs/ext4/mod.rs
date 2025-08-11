mod fs;
mod inode;
mod util;

#[allow(unused_imports)]
use axdriver::{AxBlockDevice, prelude::BlockDriverOps};
pub use fs::*;
pub use inode::*;
use lwext4_rust::{BlockDevice, EXT4_DEV_BSIZE, Ext4Error, Ext4Result, ffi::EIO};

pub(crate) struct Ext4Disk(AxBlockDevice);

impl BlockDevice for Ext4Disk {
    fn read_blocks(&mut self, block_id: u64, buf: &mut [u8]) -> Ext4Result<usize> {
        let mut block_buf = [0u8; EXT4_DEV_BSIZE];
        for (i, block) in buf.chunks_mut(EXT4_DEV_BSIZE).enumerate() {
            self.0
                .read_block(block_id + i as u64, &mut block_buf)
                .map_err(|_| Ext4Error::new(EIO as _, None))?;
            block.copy_from_slice(&block_buf);
        }
        Ok(buf.len())
    }

    fn write_blocks(&mut self, block_id: u64, buf: &[u8]) -> Ext4Result<usize> {
        let mut block_buf = [0u8; EXT4_DEV_BSIZE];
        for (i, block) in buf.chunks(EXT4_DEV_BSIZE).enumerate() {
            block_buf.copy_from_slice(block);
            self.0
                .write_block(block_id + i as u64, &block_buf)
                .map_err(|_| Ext4Error::new(EIO as _, None))?;
        }
        Ok(buf.len())
    }

    fn num_blocks(&self) -> Ext4Result<u64> {
        Ok(self.0.num_blocks())
    }
}
