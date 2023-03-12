pub use axdisplay;

use driver_display::DisplayDriverOps;
pub use driver_display::DisplayInfo;

pub fn framebuffer_info() -> DisplayInfo {
    let mut device = axdisplay::display_devices().inner.lock();
    let info = device.0.info();
    debug!(
        "[kernel] FrameBuffer: addr 0x{:X}, len {}",
        info.fb_base_vaddr, info.fb_size
    );
    info
}

pub fn framebuffer_flush() -> isize {
    let mut device = axdisplay::display_devices().inner.lock();
    device.0.flush().unwrap();
    0
}
