use crate::BlockDevice;
use driver_block::BlockDriverOps;
use driver_common::DevResult;

const BLOCK_SIZE: usize = 512;

pub struct Disk {
    block_id: u64,
    offset: usize,
    dev: BlockDevice,
}

impl Disk {
    /// Create a new disk.
    pub fn new(dev: BlockDevice) -> Self {
        assert_eq!(BLOCK_SIZE, dev.block_size());
        Self {
            block_id: 0,
            offset: 0,
            dev,
        }
    }

    /// Get the size of the disk.
    pub fn size(&self) -> u64 {
        self.dev.num_blocks() * BLOCK_SIZE as u64
    }

    /// Get the position of the cursor.
    pub fn position(&self) -> u64 {
        self.block_id * BLOCK_SIZE as u64 + self.offset as u64
    }

    /// Set the position of the cursor.
    pub fn set_position(&mut self, pos: u64) {
        self.block_id = pos / BLOCK_SIZE as u64;
        self.offset = pos as usize % BLOCK_SIZE;
    }

    /// Read within one block, returns the number of bytes read.
    pub fn read_one(&mut self, buf: &mut [u8]) -> DevResult<usize> {
        let read_size = if self.offset == 0 && buf.len() >= BLOCK_SIZE {
            // whole block
            self.dev
                .read_block(self.block_id, &mut buf[0..BLOCK_SIZE])?;
            BLOCK_SIZE
        } else {
            // partial block
            let mut data = [0u8; 512];
            self.dev.read_block(self.block_id, &mut data)?;
            let start = self.offset;
            let end = (self.offset + buf.len()).min(BLOCK_SIZE);
            buf.copy_from_slice(&data[start..end]);
            self.offset += end - start;
            if self.offset >= BLOCK_SIZE {
                self.block_id += 1;
                self.offset -= BLOCK_SIZE;
            }
            end - start
        };
        Ok(read_size)
    }

    /// Write within one block, returns the number of bytes written.
    pub fn write_one(&mut self, _buf: &[u8]) -> DevResult<usize> {
        Ok(0) // TODO
    }
}
