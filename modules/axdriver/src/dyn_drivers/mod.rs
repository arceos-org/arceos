use core::{error::Error, ops::Deref, ptr::NonNull};

use alloc::{boxed::Box, format, string::ToString};
use axerrno::{AxError, AxResult};
use axhal::mem::{PhysAddr, VirtAddr, phys_to_virt};
use lazyinit::LazyInit;
use memory_addr::MemoryAddr;
use rdrive::register::{DriverRegister, DriverRegisterSlice};

mod intc;

#[cfg(feature = "block")]
pub mod blk;

/// A function type that maps a physical address to a virtual address. map flags should be read/write/device.
pub type IoMapFunc = fn(PhysAddr, usize) -> AxResult<VirtAddr>;

static IO_MAP_FUNC: LazyInit<IoMapFunc> = LazyInit::new();

/// Sets up the device driver subsystem.
pub fn setup(dtb: usize, io_map_func: IoMapFunc) {
    IO_MAP_FUNC.init_once(io_map_func);
    if dtb == 0 {
        warn!("Device tree base address is 0, skipping device driver setup.");
        return;
    }

    let dtb_virt = phys_to_virt(dtb.into());
    if let Some(dtb) = NonNull::new(dtb_virt.as_mut_ptr()) {
        rdrive::init(rdrive::Platform::Fdt { addr: dtb }).unwrap();
        rdrive::register_append(&driver_registers());
        rdrive::probe_pre_kernel().unwrap();
    }
}

#[allow(unused)]
/// maps a mmio physical address to a virtual address.
fn iomap(addr: PhysAddr, size: usize) -> Result<NonNull<u8>, Box<dyn Error>> {
    let end = (addr + size).align_up_4k();
    let start = addr.align_down_4k();
    let offset = addr - start;
    let size = end - start;
    let iomap = *IO_MAP_FUNC
        .get()
        .ok_or_else(|| "IO map function not initialized".to_string())?;

    let virt = match iomap(start, size) {
        Ok(val) => val,
        Err(AxError::AlreadyExists) => phys_to_virt(start),
        Err(e) => {
            return Err(format!(
                "Failed to map MMIO region: {e:?} (addr: {start:?}, size: {size:#x})"
            )
            .into());
        }
    };
    let start_virt = virt + offset;
    Ok(unsafe { NonNull::new_unchecked(start_virt.as_mut_ptr()) })
}

fn driver_registers() -> impl Deref<Target = [DriverRegister]> {
    unsafe extern "C" {
        fn __sdriver_register();
        fn __edriver_register();
    }

    unsafe {
        let len = __edriver_register as usize - __sdriver_register as usize;

        if len == 0 {
            return DriverRegisterSlice::empty();
        }

        let data = core::slice::from_raw_parts(__sdriver_register as _, len);

        DriverRegisterSlice::from_raw(data)
    }
}
