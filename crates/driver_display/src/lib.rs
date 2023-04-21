//! Common traits and types for graphics display device drivers.

#![no_std]

#[doc(no_inline)]
pub use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};

/// The information of the graphics device.
#[derive(Debug, Clone, Copy)]
pub struct DisplayInfo {
    /// The visible width.
    pub width: u32,
    /// The visible height.
    pub height: u32,
    /// The base virtual address of the framebuffer.
    pub fb_base_vaddr: usize,
    /// The size of the framebuffer in bytes.
    pub fb_size: usize,
}

/// The framebuffer.
///
/// It's a special memory buffer that mapped from the device memory.
pub struct FrameBuffer<'a> {
    _raw: &'a mut [u8],
}

impl<'a> FrameBuffer<'a> {
    /// Use the given raw pointer and size as the framebuffer.
    ///
    /// # Safety
    ///
    /// Caller must insure that the given memory region is valid and accessible.
    pub unsafe fn from_raw_parts_mut(ptr: *mut u8, len: usize) -> Self {
        Self {
            _raw: core::slice::from_raw_parts_mut(ptr, len),
        }
    }

    /// Use the given slice as the framebuffer.
    pub fn from_slice(slice: &'a mut [u8]) -> Self {
        Self { _raw: slice }
    }
}

/// Operations that require a graphics device driver to implement.
pub trait DisplayDriverOps: BaseDriverOps {
    /// Get the display information.
    fn info(&self) -> DisplayInfo;

    /// Get the framebuffer.
    fn fb(&self) -> FrameBuffer;

    /// Whether need to flush the framebuffer to the screen.
    fn need_flush(&self) -> bool;

    /// Flush framebuffer to the screen.
    fn flush(&mut self) -> DevResult;
}
