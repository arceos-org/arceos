#![no_std]

use driver_common::{BaseDriverOps, DevResult};

pub trait BlockDriverOps: BaseDriverOps {
    fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> DevResult;
    fn write_block(&mut self, block_id: usize, buf: &[u8]) -> DevResult;
    fn flush(&mut self) -> DevResult;
}
