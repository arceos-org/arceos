#![no_std]
#![no_main]

extern crate libax;
use libax::gpu::display::*;

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, Rectangle, Triangle,
    },
    text::{Alignment, Text},
};

const INIT_X: i32 = 80;
const INIT_Y: i32 = 400;
const RECT_SIZE: u32 = 150;

pub struct DrawingBoard {
    disp: Display,
    latest_pos: Point,
}

impl DrawingBoard {
    pub fn new() -> Self {
        Self {
            disp: Display::new(Size::new(VIRTGPU_XRES, VIRTGPU_YRES)),
            latest_pos: Point::new(INIT_X, INIT_Y),
        }
    }
    fn paint(&mut self) {
        Rectangle::with_center(self.latest_pos, Size::new(RECT_SIZE, RECT_SIZE))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 10))
            .draw(&mut self.disp)
            .ok();
        Circle::new(self.latest_pos + Point::new(-70, -300), 150)
            .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
            .draw(&mut self.disp)
            .ok();
        Triangle::new(self.latest_pos + Point::new(0, 150), self.latest_pos + Point::new(80, 200), self.latest_pos + Point::new(-120, 300))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 10))
            .draw(&mut self.disp)
            .ok();
        let text = "ArceOS";
        Text::with_alignment(
            text,
            self.latest_pos + Point::new(0, 300),
            MonoTextStyle::new(&FONT_10X20, Rgb888::YELLOW),
            Alignment::Center,
        )
        .draw(&mut self.disp).ok();
    }
    fn unpaint(&mut self) {
        Rectangle::with_center(self.latest_pos, Size::new(RECT_SIZE, RECT_SIZE))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::BLACK, 10))
            .draw(&mut self.disp)
            .ok();
    }
    pub fn move_rect(&mut self, dx: i32, dy: i32) {
        self.unpaint();
        self.latest_pos.x += dx;
        self.latest_pos.y += dy;
        self.paint();
    }
}

fn test_gpu() -> i32 {
    let mut board = DrawingBoard::new();
    let _ = board.disp.clear(Rgb888::BLACK).unwrap();
    for _ in 0..5 {
        board.latest_pos.x += RECT_SIZE as i32 + 20;
        board.paint();
    }
    0
}


fn test_gpu_simple() -> ! {
    let mut disp = Display::new(Size::new(VIRTGPU_XRES, VIRTGPU_YRES));
    disp.paint_on_framebuffer(|fb| {
        for y in 0..VIRTGPU_YRES as usize {
            for x in 0..VIRTGPU_XRES as usize {
                let idx = (y * VIRTGPU_XRES as usize + x) * 4;
                fb[idx] = x as u8;
                fb[idx + 1] = y as u8;
                fb[idx + 2] = (x + y) as u8;
            }
        }
    });
    loop{};
}

#[no_mangle]
fn main() {
    test_gpu();
    test_gpu_simple();
}
