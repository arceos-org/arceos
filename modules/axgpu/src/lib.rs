#![no_std]

#[macro_use]
extern crate log;

use axsync::Mutex;
use lazy_init::LazyInit;
use axdriver::GpuDevices;

pub struct GpuDevicesWrapper{
    pub inner: Mutex<GpuDevices>
}

impl GpuDevicesWrapper{
    fn new(inner:Mutex<GpuDevices>) -> Self{
        GpuDevicesWrapper{inner: inner}
    }
}

static GPUS: LazyInit<GpuDevicesWrapper> = LazyInit::new();

pub fn init_gpu(gpu_devs: GpuDevices) {
    info!("Initialize Gpu subsystem...");

    info!("number of Gpus: {}", gpu_devs.len());
    GPUS.init_by(GpuDevicesWrapper::new(Mutex::new(gpu_devs)));
}

pub fn gpu_devices() -> &'static GpuDevicesWrapper {
    &GPUS
}