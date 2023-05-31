//! [ArceOS](https://github.com/rcore-os/arceos) graphics module.
//!
//! Currently only supports direct writing to the framebuffer.

#![no_std]

#[macro_use]
extern crate log;

#[doc(no_inline)]
pub use driver_display::DisplayInfo;

use axdriver::{prelude::*, AxDeviceContainer};
use axsync::Mutex;
use lazy_init::LazyInit;

static MAIN_DISPLAY: LazyInit<Mutex<AxDisplayDevice>> = LazyInit::new();

/// Initializes the graphics subsystem by underlayer devices.
pub fn init_display(mut display_devs: AxDeviceContainer<AxDisplayDevice>) {
    info!("Initialize graphics subsystem...");

    let dev = display_devs.take_one().expect("No graphics device found!");
    info!("  use graphics device 0: {:?}", dev.device_name());
    MAIN_DISPLAY.init_by(Mutex::new(dev));
}

/// Gets the framebuffer information.
pub fn framebuffer_info() -> DisplayInfo {
    MAIN_DISPLAY.lock().info()
}

/// Flushes the framebuffer, i.e. show on the screen.
pub fn framebuffer_flush() {
    MAIN_DISPLAY.lock().flush().unwrap();
}
