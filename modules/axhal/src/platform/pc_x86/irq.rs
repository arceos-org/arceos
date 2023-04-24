/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 256;

/// The timer IRQ number.
pub const TIMER_IRQ_NUM: usize = 0;

/// Enables or disables the given IRQ.
pub fn set_enable(irq_num: usize, enabled: bool) {}

/// Registers an IRQ handler for the given IRQ.
pub fn register_handler(irq_num: usize, handler: crate::irq::IrqHandler) -> bool {
    false
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(irq_num: usize) {}
