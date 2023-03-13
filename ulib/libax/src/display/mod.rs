use axdisplay;

pub use axdisplay::DisplayInfo;

pub fn framebuffer_info() -> DisplayInfo {
    axdisplay::framebuffer_info()
}

pub fn framebuffer_flush() -> isize {
    axdisplay::framebuffer_flush()
}
