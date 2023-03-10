#![no_std]

use driver_common::{BaseDriverOps, DevResult};

pub trait GpuDriverOps: BaseDriverOps {
    fn get_framebuffer(&mut self) -> &mut [u8];
    fn flush(&mut self) -> DevResult;
    fn update_cursor(&mut self) -> DevResult;
    fn get_resolution(&mut self) -> (u32, u32);
}
