use core::arch::asm;

/// Interrupt Enable Flag (IF)
const IF_BIT: usize = 1 << 9;

#[inline]
pub fn local_irq_save_and_disable() -> usize {
    let flags: usize;
    unsafe { asm!("pushf; pop {}; cli", out(reg) flags) };
    flags & IF_BIT
}

#[inline]
pub fn local_irq_restore(flags: usize) {
    if flags != 0 {
        unsafe { asm!("sti") };
    } else {
        unsafe { asm!("cli") };
    }
}
