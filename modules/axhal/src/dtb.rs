//! DTB (Device Tree Blob) related functionality.
use core::ptr::NonNull;

use fdt_parser::Fdt;
use spin::{Lazy, Once};

static BOOTARG: Once<usize> = Once::new();

/// Returns the physical address to probe for DTB.
fn dtb_paddr_from_boot_context() -> Option<usize> {
    let arg = get_bootarg();
    if arg != 0 {
        return Some(arg);
    }

    #[cfg(target_arch = "aarch64")]
    {
        /// Why fallback is needed:
        /// - On QEMU `virt`, when booting with Linux kernel boot protocol
        ///   (non-ELF passed to `-kernel`), DTB address is passed in register
        ///   (`x0` on AArch64).
        /// - For "bare-metal" boot paths (for example ELF passed to `-kernel`),
        ///   QEMU documentation states DTB is placed at start of RAM.
        ///
        /// Ref:
        /// https://www.qemu.org/docs/master/system/arm/virt.html#hardware-configuration-information-for-bare-metal-programming
        if axconfig::PLATFORM == "aarch64-qemu-virt" {
            return Some(axconfig::plat::PHYS_MEMORY_BASE);
        }
    }

    None
}

/// Initializes the boot argument.
pub fn init(arg: usize) {
    BOOTARG.call_once(|| arg);
}

/// Returns the boot argument.
/// This is typically the device tree blob address passed from the bootloader.
pub fn get_bootarg() -> usize {
    BOOTARG
        .get()
        .copied()
        .expect("Boot argument not initialized")
}

/// Get the FDT.
pub fn get_fdt() -> Option<&'static Fdt<'static>> {
    static CACHED_FDT: Lazy<Option<Fdt<'static>>> = Lazy::new(|| {
        let fdt_paddr = dtb_paddr_from_boot_context()?;
        let fdt_ptr = NonNull::new(crate::mem::phys_to_virt(fdt_paddr.into()).as_mut_ptr())?;
        Fdt::from_ptr(fdt_ptr).ok()
    });

    CACHED_FDT.as_ref()
}

/// Get the bootargs chosen from the device tree.
pub fn get_chosen_bootargs() -> Option<&'static str> {
    static CACHED_BOOTARGS: Lazy<Option<&'static str>> = Lazy::new(|| {
        let fdt = get_fdt()?;
        fdt.chosen()?.bootargs()
    });

    *CACHED_BOOTARGS
}
