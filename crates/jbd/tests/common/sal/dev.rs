use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
};

use jbd::sal::BlockDevice;

pub struct FileDevice(RefCell<File>);

pub const BLOCK_SIZE: usize = 512;

impl FileDevice {
    pub fn new(path: &str, nblocks: usize) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        file.set_len((nblocks * BLOCK_SIZE) as u64)?;
        Ok(Self(RefCell::new(file)))
    }

    pub fn with_existing(file: File) -> Self {
        Self(RefCell::new(file))
    }
}

impl BlockDevice for FileDevice {
    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let block_size = self.block_size();
        let file = &mut self.0.borrow_mut();
        file.seek(SeekFrom::Start((block_id * block_size) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), block_size, "Not a complete block!");
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let block_size = self.block_size();
        let file = &mut self.0.borrow_mut();
        file.seek(SeekFrom::Start((block_id * block_size) as u64))
            .expect("Error when seeking!");
        assert_eq!(
            file.write(buf).unwrap(),
            block_size,
            "Not a complete block!"
        );
    }
}
