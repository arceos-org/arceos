//! [ArceOS](https://github.com/rcore-os/arceos) graphics module.
//!
//! Currently only supports direct writing to the framebuffer.

#![no_std]

#[macro_use]
extern crate log;

#[doc(no_inline)]
pub use driver_display::DisplayInfo;

use axdriver::DisplayDevices;
use axsync::Mutex;
use driver_display::{BaseDriverOps, DeviceType, DisplayDriverOps};
use lazy_init::LazyInit;

struct DisplayDevicesWrapper(Mutex<DisplayDevices>);

static DISPLAYS: LazyInit<DisplayDevicesWrapper> = LazyInit::new();

/// Initializes the graphics subsystem by [`DisplayDevices`].
pub fn init_display(display_devs: DisplayDevices) {
    info!("Initialize graphics subsystem...");

    info!("number of graphics devices: {}", display_devs.len());
    axdriver::display_devices_enumerate!((i, dev) in display_devs {
        assert_eq!(dev.device_type(), DeviceType::Display);
        info!("  device {}: {:?}", i, dev.device_name());
    });
    DISPLAYS.init_by(DisplayDevicesWrapper(Mutex::new(display_devs)));
}

/// Gets the framebuffer information.
pub fn framebuffer_info() -> DisplayInfo {
    let device = DISPLAYS.0.lock();
    device.0.info()
}

/// Flushes the framebuffer, i.e. show on the screen.
pub fn framebuffer_flush() -> isize {
    let mut device = DISPLAYS.0.lock();
    device.0.flush().unwrap();
    0
}
