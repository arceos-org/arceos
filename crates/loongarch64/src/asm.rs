//! Assembly instructions

macro_rules! instruction {
    ($(#[$attr:meta])*, $fnname:ident, $asm:expr) => (
        $(#[$attr])*
        #[inline]
        pub unsafe fn $fnname() {
            match () {
                #[cfg(target_arch = "loongarch64")]
                () => core::arch::asm!($asm),

                #[cfg(not(target_arch = "loongarch64"))]
                () => unimplemented!(),
            }
        }
    )
}

instruction!(
    /// `nop` instruction wrapper
    ///
    /// Generates a no-operation.  Useful to prevent delay loops from being optimized away.
    , nop, "nop");
instruction!(
    /// `EBREAK` instruction wrapper
    ///
    /// Generates a breakpoint exception.
    , r#break, "break");
instruction!(
    /// `EBREAK` instruction wrapper
    ///
    /// The format is `idle level`. What is level is still unknown. Temporarily use `1` as `level`.
    , idle, "idle 1");
