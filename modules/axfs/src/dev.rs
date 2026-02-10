// Copyright 2025 The Axvisor Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use alloc::sync::Arc;
use axdriver::prelude::*;
use spin::Mutex;

const BLOCK_SIZE: usize = 512;

/// A disk device with a cursor.
#[derive(Clone)]
pub struct Disk {
    block_id: u64,
    offset: usize,
    dev: Arc<Mutex<AxBlockDevice>>,
}

impl Disk {
    /// Create a new disk.
    pub fn new(dev: AxBlockDevice) -> Self {
        assert_eq!(BLOCK_SIZE, dev.block_size());
        Self {
            block_id: 0,
            offset: 0,
            dev: Arc::new(Mutex::new(dev)),
        }
    }

    /// Get the size of the disk.
    pub fn size(&self) -> u64 {
        let dev = self.dev.lock();
        dev.num_blocks() * BLOCK_SIZE as u64
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
            let mut dev = self.dev.lock();
            dev.read_block(self.block_id, &mut buf[0..BLOCK_SIZE])?;
            self.block_id += 1;
            BLOCK_SIZE
        } else {
            // partial block
            let mut data = [0u8; BLOCK_SIZE];
            let start = self.offset;
            let count = buf.len().min(BLOCK_SIZE - self.offset);

            {
                let mut dev = self.dev.lock();
                dev.read_block(self.block_id, &mut data)?;
            }
            buf[..count].copy_from_slice(&data[start..start + count]);
            self.offset += count;
            if self.offset >= BLOCK_SIZE {
                self.block_id += 1;
                self.offset -= BLOCK_SIZE;
            }
            count
        };
        Ok(read_size)
    }

    /// Write within one block, returns the number of bytes written.
    pub fn write_one(&mut self, buf: &[u8]) -> DevResult<usize> {
        let write_size = if self.offset == 0 && buf.len() >= BLOCK_SIZE {
            // whole block
            let mut dev = self.dev.lock();
            dev.write_block(self.block_id, &buf[0..BLOCK_SIZE])?;
            self.block_id += 1;
            BLOCK_SIZE
        } else {
            // partial block
            let mut data = [0u8; BLOCK_SIZE];
            let start = self.offset;
            let count = buf.len().min(BLOCK_SIZE - self.offset);

            let mut dev = self.dev.lock();
            dev.read_block(self.block_id, &mut data)?;
            data[start..start + count].copy_from_slice(&buf[..count]);
            dev.write_block(self.block_id, &data)?;

            self.offset += count;
            if self.offset >= BLOCK_SIZE {
                self.block_id += 1;
                self.offset -= BLOCK_SIZE;
            }
            count
        };
        Ok(write_size)
    }
}

/// A partition wrapper that provides access to a specific partition of a disk.
pub struct Partition {
    disk: Arc<Mutex<Disk>>,
    start_lba: u64,
    end_lba: u64,
    position: u64,
}

impl Partition {
    /// Create a new partition wrapper.
    pub fn new(disk: Disk, start_lba: u64, end_lba: u64) -> Self {
        Self {
            disk: Arc::new(Mutex::new(disk)),
            start_lba,
            end_lba,
            position: 0,
        }
    }

    /// Get the size of the partition.
    pub fn size(&self) -> u64 {
        (self.end_lba - self.start_lba + 1) * BLOCK_SIZE as u64
    }

    /// Get the position of the cursor.
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Set the position of the cursor.
    pub fn set_position(&mut self, pos: u64) {
        self.position = pos.min(self.size());
    }

    /// Read within one block, returns the number of bytes read.
    pub fn read_one(&mut self, buf: &mut [u8]) -> DevResult<usize> {
        if self.position >= self.size() {
            return Ok(0);
        }

        let remaining = self.size() - self.position;
        let to_read = buf.len().min(remaining as usize);
        let buf = &mut buf[..to_read];

        // Calculate the absolute position on the disk
        let abs_pos = self.start_lba * BLOCK_SIZE as u64 + self.position;

        // Set disk position and read
        let read_len = {
            let mut disk = self.disk.lock();
            disk.set_position(abs_pos);
            disk.read_one(buf)?
        };

        self.position += read_len as u64;
        Ok(read_len)
    }

    /// Write within one block, returns the number of bytes written.
    pub fn write_one(&mut self, buf: &[u8]) -> DevResult<usize> {
        if self.position >= self.size() {
            return Ok(0);
        }

        let remaining = self.size() - self.position;
        let to_write = buf.len().min(remaining as usize);
        let buf = &buf[..to_write];

        // Calculate the absolute position on the disk
        let abs_pos = self.start_lba * BLOCK_SIZE as u64 + self.position;

        // Set disk position and write
        let write_len = {
            let mut disk = self.disk.lock();
            disk.set_position(abs_pos);
            disk.write_one(buf)?
        };

        self.position += write_len as u64;
        Ok(write_len)
    }
}
