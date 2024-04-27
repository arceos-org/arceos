use core::arch::asm;

/// Bit 2: Supervisor Interrupt Enable
const IE_BIT: usize = 1 << 2;

#[inline]
pub fn local_irq_save_and_disable() -> usize {
    let mut flags: usize = 0;
    // clear the `IE` bit, and return the old CSR
    unsafe { asm!("csrxchg {}, {}, 0x0", inout(reg) flags, in(reg) IE_BIT) };
    flags & IE_BIT
}

#[inline]
pub fn local_irq_restore(mut flags: usize) {
    // restore the `IE` bit
    unsafe { asm!("csrxchg {}, {}, 0x0", inout(reg) flags, in(reg) IE_BIT) };
}
