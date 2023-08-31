//! Mock block devices that store data in RAM.

extern crate alloc;

use crate::BlockDriverOps;
use alloc::{vec, vec::Vec};
use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};

const BLOCK_SIZE: usize = 512;

/// A RAM disk that stores data in a vector.
#[derive(Default)]
pub struct RamDisk {
    size: usize,
    data: Vec<u8>,
}

impl RamDisk {
    /// Creates a new RAM disk with the given size hint.
    ///
    /// The actual size of the RAM disk will be aligned upwards to the block
    /// size (512 bytes).
    pub fn new(size_hint: usize) -> Self {
        let size = align_up(size_hint);
        Self {
            size,
            data: vec![0; size],
        }
    }

    /// Creates a new RAM disk from the exiting data.
    ///
    /// The actual size of the RAM disk will be aligned upwards to the block
    /// size (512 bytes).
    pub fn from(buf: &[u8]) -> Self {
        let size = align_up(buf.len());
        let mut data = vec![0; size];
        data[..buf.len()].copy_from_slice(buf);
        Self { size, data }
    }

    /// Returns the size of the RAM disk in bytes.
    pub const fn size(&self) -> usize {
        self.size
    }
}

impl const BaseDriverOps for RamDisk {
    fn device_type(&self) -> DeviceType {
        DeviceType::Block
    }

    fn device_name(&self) -> &str {
        "ramdisk"
    }
}

impl BlockDriverOps for RamDisk {
    #[inline]
    fn num_blocks(&self) -> u64 {
        (self.size / BLOCK_SIZE) as u64
    }

    #[inline]
    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    fn read_block(&mut self, block_id: u64, buf: &mut [u8]) -> DevResult {
        let offset = block_id as usize * BLOCK_SIZE;
        if offset + buf.len() > self.size {
            return Err(DevError::Io);
        }
        if buf.len() % BLOCK_SIZE != 0 {
            return Err(DevError::InvalidParam);
        }
        buf.copy_from_slice(&self.data[offset..offset + buf.len()]);
        Ok(())
    }

    fn write_block(&mut self, block_id: u64, buf: &[u8]) -> DevResult {
        let offset = block_id as usize * BLOCK_SIZE;
        if offset + buf.len() > self.size {
            return Err(DevError::Io);
        }
        if buf.len() % BLOCK_SIZE != 0 {
            return Err(DevError::InvalidParam);
        }
        self.data[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(())
    }

    fn flush(&mut self) -> DevResult {
        Ok(())
    }
}

const fn align_up(val: usize) -> usize {
    (val + BLOCK_SIZE - 1) & !(BLOCK_SIZE - 1)
}
