use core::arch::asm;

/// Bit 1: Supervisor Interrupt Enable
const SIE_BIT: usize = 1 << 1;

#[inline]
pub fn local_irq_save_and_disable() -> usize {
    let flags: usize;
    // clear the `SIE` bit, and return the old CSR
    unsafe { asm!("csrrc {}, sstatus, {}", out(reg) flags, const SIE_BIT) };
    flags & SIE_BIT
}

#[inline]
pub fn local_irq_restore(flags: usize) {
    // restore the `SIE` bit
    unsafe { asm!("csrrs x0, sstatus, {}", in(reg) flags) };
}
