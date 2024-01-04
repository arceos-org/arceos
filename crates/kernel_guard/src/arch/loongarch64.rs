use core::arch::asm;

#[inline]
pub fn local_irq_save_and_disable() -> usize {
    let mut flags: usize = 0;
    let ie_mask: usize = 1 << 2;
    // clear the `IE` bit, and return the old CSR
    // unsafe { asm!("csrrd {}, 0x0", out(reg) flags) };
    unsafe { asm!("csrxchg {}, {}, 0x0", inout(reg)flags, in(reg) ie_mask) };
    flags & ie_mask
}

#[inline]
#[allow(unused_assignments)]
pub fn local_irq_restore(mut flags: usize) {
    // restore the `IE` bit
    let mask: usize = 1 << 2;
    unsafe { asm!("csrxchg {}, {}, 0x0", inout(reg)flags, in(reg) mask) };
}
