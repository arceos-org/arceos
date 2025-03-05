use crate::mem::phys_to_virt;
use memory_addr::pa;

const HALT_ADDR: *mut u8 = phys_to_virt(pa!(axconfig::devices::GED_PADDR)).as_mut_ptr();

/// Shutdown the whole system, including all CPUs.
pub fn terminate() -> ! {
    info!("Shutting down...");
    unsafe { HALT_ADDR.write_volatile(0x34) };
    crate::arch::halt();
    warn!("It should shutdown!");
    loop {
        crate::arch::halt();
    }
}
