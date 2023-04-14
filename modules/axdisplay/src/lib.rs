#![no_std]

#[macro_use]
extern crate log;

#[doc(no_inline)]
pub use driver_display::DisplayInfo;

use axdriver::DisplayDevices;
use axsync::Mutex;
use driver_display::DisplayDriverOps;
use lazy_init::LazyInit;

struct DisplayDevicesWrapper(Mutex<DisplayDevices>);

static DISPLAYS: LazyInit<DisplayDevicesWrapper> = LazyInit::new();

pub fn init_display(display_devs: DisplayDevices) {
    info!("Initialize Display subsystem...");

    info!("number of Displays: {}", display_devs.len());
    DISPLAYS.init_by(DisplayDevicesWrapper(Mutex::new(display_devs)));
}

pub fn framebuffer_info() -> DisplayInfo {
    let device = DISPLAYS.0.lock();
    device.0.info()
}

pub fn framebuffer_flush() -> isize {
    let mut device = DISPLAYS.0.lock();
    device.0.flush().unwrap();
    0
}
