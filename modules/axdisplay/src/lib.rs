//! [ArceOS](https://github.com/arceos-org/arceos) graphics module.
//!
//! Currently only supports direct writing to the framebuffer.

#![no_std]

#[macro_use]
extern crate log;

use axdriver::{AxDeviceContainer, prelude::*};
use axsync::{Mutex, MutexGuard};
use lazyinit::LazyInit;

static MAIN_DISPLAY: LazyInit<Mutex<AxDisplayDevice>> = LazyInit::new();

/// Initializes the graphics subsystem by underlayer devices.
pub fn init_display(mut display_devs: AxDeviceContainer<AxDisplayDevice>) {
    info!("Initialize graphics subsystem...");

    if let Some(dev) = display_devs.take_one() {
        info!("  use graphics device 0: {:?}", dev.device_name());
        MAIN_DISPLAY.init_once(Mutex::new(dev));
    }
}

/// Checks if there is a display device.
pub fn has_display() -> bool {
    MAIN_DISPLAY.is_inited()
}

/// Gets the framebuffer information.
pub fn main_display() -> MutexGuard<'static, AxDisplayDevice> {
    MAIN_DISPLAY.lock()
}
