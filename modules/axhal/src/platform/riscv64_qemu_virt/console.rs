use core::ptr::addr_of;

use memory_addr::VirtAddr;

use crate::mem::virt_to_phys;

/// Writes a byte to the console.
pub fn putchar(c: u8) {
    sbi_rt::console_write_byte(c);
}

/// Reads a byte from the console, or returns [`None`] if no input is available.
pub fn getchar() -> Option<u8> {
    let c: u8 = 0;
    sbi_rt::console_read(sbi_rt::Physical::new(
        1,
        virt_to_phys(VirtAddr::from_ptr_of(addr_of!(c))).as_usize(),
        0,
    ))
    .ok()
    .map(|_| c)
}
