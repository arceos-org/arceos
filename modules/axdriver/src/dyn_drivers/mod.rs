use alloc::vec::Vec;
use core::{ops::Deref, ptr::NonNull};

use axerrno::AxError;
use axhal::mem::{PhysAddr, phys_to_virt};
use rdrive::{
    DeviceId, IrqConfig,
    probe::OnProbeError,
    register::{DriverRegister, DriverRegisterSlice},
};

mod cache;
mod pci;

#[cfg(feature = "block")]
pub mod blk;

#[allow(dead_code)]
/// maps a mmio physical address to a virtual address.
fn iomap(addr: PhysAddr, size: usize) -> Result<NonNull<u8>, OnProbeError> {
    axklib::mem::iomap(addr, size)
        .map_err(|e| match e {
            AxErrorKind::NoMemory => OnProbeError::KError(rdrive::KError::NoMem),
            _ => OnProbeError::Other(alloc::format!("{e:?}").into()),
        })
        .map(|v| unsafe { NonNull::new_unchecked(v.as_mut_ptr()) })
}

fn driver_registers() -> impl Deref<Target = [DriverRegister]> {
    unsafe extern "C" {
        fn _sdriver();
        fn _edriver();
    }

    unsafe {
        let len = _edriver as *const () as usize - _sdriver as *const () as usize;

        if len == 0 {
            return DriverRegisterSlice::empty();
        }

        let data = core::slice::from_raw_parts(_sdriver as _, len);

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
