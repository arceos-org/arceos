#![no_std]
#![feature(doc_auto_cfg)]

#[cfg(feature = "ramdisk")]
pub mod ramdisk;

use driver_common::{BaseDriverOps, DevResult};

pub trait BlockDriverOps: BaseDriverOps {
    fn num_blocks(&self) -> u64;
    fn block_size(&self) -> usize;

    fn read_block(&mut self, block_id: u64, buf: &mut [u8]) -> DevResult;
    fn write_block(&mut self, block_id: u64, buf: &[u8]) -> DevResult;
    fn flush(&mut self) -> DevResult;
}
