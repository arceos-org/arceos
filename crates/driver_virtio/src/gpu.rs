extern crate alloc;
use crate::as_dev_err;

use driver_common::{BaseDriverOps, DevResult, DeviceType};
use driver_display::{DisplayDriverOps, DisplayInfo, FrameBuffer};
use virtio_drivers::{device::gpu::VirtIOGpu as InnerDev, transport::Transport, Hal};

/// The VirtIO GPU device driver.
pub struct VirtIoGpuDev<H: Hal, T: Transport> {
    inner: InnerDev<'static, H, T>,
    info: DisplayInfo,
}

unsafe impl<H: Hal, T: Transport> Send for VirtIoGpuDev<H, T> {}
unsafe impl<H: Hal, T: Transport> Sync for VirtIoGpuDev<H, T> {}

impl<H: Hal, T: Transport> VirtIoGpuDev<H, T> {
    /// Creates a new driver instance and initializes the device, or returns
    /// an error if any step fails.
    pub fn try_new(transport: T) -> DevResult<Self> {
        let mut virtio = InnerDev::new(transport).unwrap();

        // get framebuffer
        let fbuffer = virtio.setup_framebuffer().unwrap();
        let fb_base_vaddr = fbuffer.as_mut_ptr() as usize;
        let fb_size = fbuffer.len();
        let (width, height) = virtio.resolution().unwrap();
        let info = DisplayInfo {
            width,
            height,
            fb_base_vaddr,
            fb_size,
        };

        Ok(Self {
            inner: virtio,
            info,
        })
    }
}

impl<H: Hal, T: Transport> const BaseDriverOps for VirtIoGpuDev<H, T> {
    fn device_name(&self) -> &str {
        "virtio-gpu"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Display
    }
}

impl<H: Hal, T: Transport> DisplayDriverOps for VirtIoGpuDev<H, T> {
    fn info(&self) -> DisplayInfo {
        self.info
    }

    fn fb(&self) -> FrameBuffer {
        unsafe {
            FrameBuffer::from_raw_parts_mut(self.info.fb_base_vaddr as *mut u8, self.info.fb_size)
        }
    }

    fn need_flush(&self) -> bool {
        true
    }

    fn flush(&mut self) -> DevResult {
        self.inner.flush().map_err(as_dev_err)
    }
}
