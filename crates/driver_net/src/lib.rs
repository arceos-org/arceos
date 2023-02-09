#![no_std]

use driver_common::{BaseDriverOps, DevResult};

pub trait NetDriverOps: BaseDriverOps {
    fn send(&self, buf: &[u8]) -> DevResult<usize>;
    fn recv(&self, buf: &mut [u8]) -> DevResult<usize>;
}
