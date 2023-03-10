extern crate alloc;
use crate::as_dev_err;
use alloc::vec::Vec;

use driver_gpu::GpuDriverOps;
use driver_common::{BaseDriverOps, DevResult, DeviceType};
use virtio_drivers::{device::gpu::VirtIOGpu as InnerDev, transport::Transport, Hal};

use tinybmp::Bmp;
use embedded_graphics::pixelcolor::Rgb888;
static BMP_DATA: &[u8] = include_bytes!("../images/mouse.bmp");

pub struct VirtIoGpuDev<H: Hal, T: Transport> {
    pub inner: InnerDev<'static, H, T>,
    pub fb: &'static [u8],
}

unsafe impl<H: Hal, T: Transport> Send for VirtIoGpuDev<H, T> {}
unsafe impl<H: Hal, T: Transport> Sync for VirtIoGpuDev<H, T> {}

impl<H: Hal, T: Transport> VirtIoGpuDev<H, T> {
    pub fn try_new(transport: T) -> DevResult<Self> {
        unsafe{
            let mut virtio = 
                InnerDev::new(transport).unwrap();
            
            // get framebuffer
            let fbuffer = virtio.setup_framebuffer().unwrap();
            let ptr = fbuffer.as_mut_ptr();
            let len = fbuffer.len();
            let fb = core::slice::from_raw_parts_mut(ptr, len);

            let bmp = Bmp::<Rgb888>::from_slice(BMP_DATA).unwrap();
            let raw = bmp.as_raw();
            let mut b = Vec::new();
            for i in raw.image_data().chunks(3) {
                let mut v = i.to_vec();
                b.append(&mut v);
                if i == [255, 255, 255] {
                    b.push(0x0)
                } else {
                    b.push(0xff)
                }
            }
            virtio.setup_cursor(b.as_slice(), 50, 50, 50, 50).unwrap();

            Ok(Self {
                inner: virtio,
                fb,
            })
        }
    }
}

impl<H: Hal, T: Transport> const BaseDriverOps for VirtIoGpuDev<H, T> {
    fn device_name(&self) -> &str {
        "virtio-gpu"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Gpu
    }
}

impl<H: Hal, T: Transport> GpuDriverOps for VirtIoGpuDev<H, T> {
    fn get_framebuffer(&mut self) -> &mut [u8] {
        unsafe{
            let ptr = self.fb.as_ptr() as *const _ as *mut u8;
            core::slice::from_raw_parts_mut(ptr, self.fb.len())
        }
    }

    // todo
    fn update_cursor(&mut self) -> DevResult {
        Ok(())
    }

    fn flush(&mut self) -> DevResult {
        self.inner.flush().map_err(as_dev_err)
    }

    fn get_resolution(&mut self) -> (u32, u32) {
        self.inner.resolution().unwrap()
    }
}
