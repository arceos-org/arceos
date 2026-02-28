//! [ArceOS](https://github.com/arceos-org/arceos) input module.

#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::vec::Vec;
use core::mem;

use axdriver::{AxDeviceContainer, prelude::*};
use axsync::Mutex;
use lazyinit::LazyInit;

static DEVICES: LazyInit<Mutex<Vec<AxInputDevice>>> = LazyInit::new();

/// Initializes the graphics subsystem by underlayer devices.
pub fn init_input(mut input_devs: AxDeviceContainer<AxInputDevice>) {
    info!("Initialize input subsystem...");

    let mut devices = Vec::new();
    while let Some(dev) = input_devs.take_one() {
        info!(
            "  registered a new {:?} input device: {}",
            dev.device_type(),
            dev.device_name(),
        );
        devices.push(dev);
    }
    DEVICES.init_once(Mutex::new(devices));
}

/// Takes the initialized input devices.
pub fn take_inputs() -> Vec<AxInputDevice> {
    mem::take(&mut DEVICES.lock())
}
