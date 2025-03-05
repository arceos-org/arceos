use loongArch64::register::{
    ecfg::{self, LineBasedInterrupt},
    estat, ticlr,
};

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 12;

/// The timer IRQ number.
pub const TIMER_IRQ_NUM: usize = estat::Interrupt::Timer as usize;

/// Enables or disables the given IRQ.
pub fn set_enable(irq_num: usize, enabled: bool) {
    if irq_num == TIMER_IRQ_NUM {
        let old_value = ecfg::read().lie();
        let new_value = match enabled {
            true => old_value | LineBasedInterrupt::TIMER,
            false => old_value & !LineBasedInterrupt::TIMER,
        };
        ecfg::set_lie(new_value);
    }
}

/// Registers an IRQ handler for the given IRQ.
pub fn register_handler(irq_num: usize, handler: crate::irq::IrqHandler) -> bool {
    crate::irq::register_handler_common(irq_num, handler)
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(irq_num: usize) {
    if irq_num == TIMER_IRQ_NUM {
        ticlr::clear_timer_interrupt();
    }
    crate::irq::dispatch_irq_common(irq_num)
}
