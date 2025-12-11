//! DTB (Device Tree Blob) related functionality.
use core::fmt::Write;
use fdt_parser::{Fdt, FdtHeader};
use lazyinit::LazyInit;

/// Record address of the boot argument (DTB address).
pub static BOOT_ARG: LazyInit<usize> = LazyInit::new();
static CACHED_FDT: LazyInit<Fdt<'static>> = LazyInit::new();
static BOOTARGS_STR: LazyInit<heapless::String<1024>> = LazyInit::new();

/// Returns the boot argument.
/// This is typically the device tree blob address passed from the bootloader.
pub fn get_dtb_address() -> usize {
    *BOOT_ARG
}

/// Get the cached FDT or initialize it if not already done.
pub fn get_fdt() -> Option<&'static Fdt<'static>> {
    // Return cached FDT if available
    if let Some(fdt) = CACHED_FDT.get() {
        return Some(fdt);
    }

    // Parse and cache the FDT
    let fdt_addr = get_dtb_address();
    if fdt_addr == 0 {
        return None;
    }

    let virt_addr = crate::mem::phys_to_virt(crate::mem::PhysAddr::from(fdt_addr)).as_usize();

    let fdt = unsafe {
        // First read the header to get the size
        let header_size = core::mem::size_of::<FdtHeader>();
        let header_ptr = virt_addr as *const u8;
        let header_slice = core::slice::from_raw_parts(header_ptr, header_size);

        let fdt_header = match FdtHeader::from_bytes(header_slice) {
            Ok(header) => header,
            Err(_) => return None,
        };

        let size = fdt_header.total_size() as usize;
        let fdt_ptr = virt_addr as *const u8;
        let fdt_slice = core::slice::from_raw_parts(fdt_ptr, size);

        let fdt_slice_static = core::mem::transmute::<&[u8], &'static [u8]>(fdt_slice);

        match Fdt::from_bytes(fdt_slice_static) {
            Ok(fdt) => fdt,
            Err(_) => return None,
        }
    };

    // Store the FDT in the cache
    CACHED_FDT.init_once(fdt);
    CACHED_FDT.get()
}

/// Get the bootargs chosen from the device tree.
pub fn get_chosen() -> Option<&'static str> {
    // If bootargs are already cached, return them
    if let Some(bootargs) = BOOTARGS_STR.get() {
        return Some(bootargs.as_str());
    }

    // Get or initialize the cached FDT
    let fdt = get_fdt()?;

    if let Some(chosen) = fdt.chosen() {
        if let Some(bootargs) = chosen.bootargs() {
            // Store bootargs in static variable
            let mut bootargs_str = heapless::String::<1024>::new();
            if write!(bootargs_str, "{}", bootargs).is_ok() {
                BOOTARGS_STR.init_once(bootargs_str);
                return Some(BOOTARGS_STR.as_str());
            } else {
                warn!("Failed to write bootargs");
            }
        }
    }
    None
}
