use core::arch::asm;

#[inline]
pub fn local_irq_save_and_disable() -> usize {
    let flags: usize;
    // save `DAIF` flags, mask `I` bit (disable IRQs)
    unsafe { asm!("mrs {}, daif; msr daifset, #2", out(reg) flags) };
    flags
}

#[inline]
pub fn local_irq_restore(flags: usize) {
    unsafe { asm!("msr daif, {}", in(reg) flags) };
}
