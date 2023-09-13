pub use axdisplay::DisplayInfo as AxDisplayInfo;

/// Gets the framebuffer information.
pub fn ax_framebuffer_info() -> AxDisplayInfo {
    axdisplay::framebuffer_info()
}

/// Flushes the framebuffer, i.e. show on the screen.
pub fn ax_framebuffer_flush() {
    axdisplay::framebuffer_flush()
}
