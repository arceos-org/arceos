use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::{RgbColor, Size};
use embedded_graphics::{draw_target::DrawTarget, prelude::OriginDimensions};

pub use libax::display::{framebuffer_flush, framebuffer_info, DisplayInfo};

pub const VIRTGPU_XRES: u32 = 1280;
pub const VIRTGPU_YRES: u32 = 800;

pub const VIRTGPU_LEN: usize = (VIRTGPU_XRES * VIRTGPU_YRES * 4) as usize;

pub struct Display {
    pub size: Size,
    pub fb: &'static mut [u8],
}

impl Display {
    pub fn new(size: Size) -> Self {
        let info = framebuffer_info();
        let fb =
            unsafe { core::slice::from_raw_parts_mut(info.fb_base_vaddr as *mut u8, VIRTGPU_LEN) };
        Self { size, fb }
    }

    pub fn framebuffer(&mut self) -> &mut [u8] {
        self.fb
    }

    pub fn paint_on_framebuffer(&mut self, p: impl FnOnce(&mut [u8])) {
        p(self.framebuffer());
        framebuffer_flush();
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        self.size
    }
}

impl DrawTarget for Display {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        pixels.into_iter().for_each(|px| {
            let idx = (px.0.y * VIRTGPU_XRES as i32 + px.0.x) as usize * 4;
            if idx + 2 >= self.fb.len() {
                return;
            }
            self.fb[idx] = px.1.b();
            self.fb[idx + 1] = px.1.g();
            self.fb[idx + 2] = px.1.r();
        });
        framebuffer_flush();
        Ok(())
    }
}