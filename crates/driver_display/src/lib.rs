#![no_std]

use driver_common::{BaseDriverOps, DevResult};

#[derive(Debug, Clone, Copy)]
pub struct DisplayInfo {
    /// visible width
    pub width: u32,
    /// visible height
    pub height: u32,
    /// frame buffer base virtual address
    pub fb_base_vaddr: usize,
    /// frame buffer size
    pub fb_size: usize,
}

#[allow(dead_code)]
pub struct FrameBuffer<'a> {
    raw: &'a mut [u8],
}

impl<'a> FrameBuffer<'a> {
    /// # Safety
    ///
    /// This function is unsafe because it created the `FrameBuffer` structure
    /// from the raw pointer.
    pub unsafe fn from_raw_parts_mut(ptr: *mut u8, len: usize) -> Self {
        Self {
            raw: core::slice::from_raw_parts_mut(ptr, len),
        }
    }

    pub fn from_slice(slice: &'a mut [u8]) -> Self {
        Self { raw: slice }
    }
}

pub trait DisplayDriverOps: BaseDriverOps {
    fn info(&self) -> DisplayInfo;
    fn fb(&self) -> FrameBuffer;
    fn need_flush(&self) -> bool;
    fn flush(&mut self) -> DevResult;
}
