#![no_std]

#[macro_use]
extern crate log;

pub use driver_display::{DisplayDriverOps, DisplayInfo};

use axdriver::DisplayDevices;
use axsync::Mutex;
use lazy_init::LazyInit;

pub struct DisplayDevicesWrapper {
    pub inner: Mutex<DisplayDevices>,
}

impl DisplayDevicesWrapper {
    fn new(inner: Mutex<DisplayDevices>) -> Self {
        DisplayDevicesWrapper { inner }
    }
}

static DISPLAYS: LazyInit<DisplayDevicesWrapper> = LazyInit::new();

pub fn init_display(display_devs: DisplayDevices) {
    info!("Initialize Display subsystem...");

    info!("number of Displays: {}", display_devs.len());
    DISPLAYS.init_by(DisplayDevicesWrapper::new(Mutex::new(display_devs)));
}

pub fn display_devices() -> &'static DisplayDevicesWrapper {
    &DISPLAYS
}
