#![no_std]

use driver_common::{BaseDriverOps, DevResult};

pub trait BlockDriverOps: BaseDriverOps {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> DevResult;
    fn write_block(&self, block_id: usize, buf: &[u8]) -> DevResult;
    fn flush(&self) -> DevResult;
}
