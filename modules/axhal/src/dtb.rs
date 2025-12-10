//! DTB (Device Tree Blob) related functionality.
use fdt_parser::{Fdt, FdtHeader};

use core::fmt::Write;
use lazyinit::LazyInit;

static BOOTARGS_STR: LazyInit<heapless::String<256>> = LazyInit::new();

/// Get the bootargs from the device tree.
pub fn bootargs_message() -> Option<&'static str> {
    let fdt_addr = crate::get_bootarg();

    if fdt_addr == 0 {
        return None;
    }

    let virt_addr = crate::mem::phys_to_virt(crate::mem::PhysAddr::from(fdt_addr)).as_usize();

    let fdt_header = unsafe {
        let header_size = core::mem::size_of::<FdtHeader>();
        let ptr = virt_addr as *const u8;
        core::slice::from_raw_parts(ptr, header_size)
    };

    let fdt_header = match FdtHeader::from_bytes(fdt_header) {
        Ok(header) => header,
        Err(_) => return None,
    };

    let fdt_bytes = unsafe {
        let ptr = virt_addr as *const u8;
        let size = fdt_header.total_size() as usize;
        core::slice::from_raw_parts(ptr, size)
    };

    let fdt = match Fdt::from_bytes(fdt_bytes) {
        Ok(fdt) => fdt,
        Err(_) => return None,
    };

    if let Some(chosen) = fdt.chosen() {
        if let Some(bootargs) = chosen.bootargs() {
            // Store bootargs in static variable
            let mut bootargs_str = heapless::String::<256>::new();
            if write!(bootargs_str, "{}", bootargs).is_ok() {
                BOOTARGS_STR.init_once(bootargs_str);
                return Some(BOOTARGS_STR.as_str());
            }
        }
    }
    None
}
