use core::{ops::Deref, ptr::NonNull};

use alloc::vec::Vec;
use axerrno::AxError;
use axhal::mem::{PhysAddr, phys_to_virt};
use rdrive::{
    DeviceId, IrqConfig,
    probe::OnProbeError,
    register::{DriverRegister, DriverRegisterSlice},
};

mod cache;
mod intc;
mod klib;
mod pci;

#[cfg(feature = "block")]
pub mod blk;

/// Sets up the device driver subsystem.
pub fn setup(dtb: usize) {
    if dtb == 0 {
        warn!("Device tree base address is 0, skipping device driver setup.");
        return;
    }
    cache::setup_dma_api();
    let dtb_virt = phys_to_virt(dtb.into());
    if let Some(dtb) = NonNull::new(dtb_virt.as_mut_ptr()) {
        rdrive::init(rdrive::Platform::Fdt { addr: dtb }).unwrap();
        rdrive::register_append(&driver_registers());
        // rdrive::probe_pre_kernel().unwrap();
    }
}

#[allow(dead_code)]
/// maps a mmio physical address to a virtual address.
fn iomap(addr: PhysAddr, size: usize) -> Result<NonNull<u8>, OnProbeError> {
    axklib::mem::iomap(addr, size)
        .map_err(|e| match e {
            AxError::NoMemory => OnProbeError::KError(rdrive::KError::NoMem),
            _ => OnProbeError::Other(alloc::format!("{e:?}").into()),
        })
        .map(|v| unsafe { NonNull::new_unchecked(v.as_mut_ptr()) })
}

#[allow(dead_code)]
fn parse_fdt_irq(intc: DeviceId, irq: &[u32]) -> IrqConfig {
    let intc = rdrive::get::<rdif_intc::Intc>(intc).expect("No interrupt controller found");
    let intc = intc.lock().unwrap();
    let fdt_parse = intc.parse_dtb_fn().expect("No DTB parse function found");
    fdt_parse(irq).unwrap()
}

fn driver_registers() -> impl Deref<Target = [DriverRegister]> {
    unsafe extern "C" {
        fn __sdriver_register();
        fn __edriver_register();
    }

    unsafe {
        let len =
            __edriver_register as *const () as usize - __sdriver_register as *const () as usize;

        if len == 0 {
            return DriverRegisterSlice::empty();
        }

        let data = core::slice::from_raw_parts(__sdriver_register as _, len);

        DriverRegisterSlice::from_raw(data)
    }
}

pub fn probe_all_devices() -> Vec<super::AxDeviceEnum> {
    rdrive::probe_all(true).unwrap();
    #[allow(unused_mut)]
    let mut devices = Vec::new();
    #[cfg(feature = "block")]
    {
        let ls = rdrive::get_list::<rdif_block::Block>();
        for dev in ls {
            devices.push(super::AxDeviceEnum::from_block(
                crate::dyn_drivers::blk::Block::from(dev),
            ));
        }
    }
    devices
}
