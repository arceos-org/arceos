//! Graphics manipulation operations.

use axdisplay;

pub use axdisplay::DisplayInfo;

/// Returns the framebuffer information.
pub fn framebuffer_info() -> DisplayInfo {
    axdisplay::framebuffer_info()
}

/// Flushes the framebuffer.
pub fn framebuffer_flush() {
    axdisplay::framebuffer_flush()
}
