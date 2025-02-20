use crate::irq::IrqHandler;
use lazyinit::LazyInit;
use loongArch64::register::{
    ecfg::{self, LineBasedInterrupt},
    ticlr,
};

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 256;

/// The Extend IRQ number.
pub const EXT_IRQ_NUM: usize = 2;

/// The timer IRQ number.
pub const TIMER_IRQ_NUM: usize = 11;

static TIMER_HANDLER: LazyInit<IrqHandler> = LazyInit::new();

macro_rules! with_cause {
    ($cause: expr, @TIMER => $timer_op: expr, @EXT => $ext_op: expr $(,)?) => {
        match $cause {
            TIMER_IRQ_NUM => $timer_op,
            EXT_IRQ_NUM => $ext_op,
            _ => panic!("invalid trap cause: {:#x}", $cause),
        }
    };
}

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
    with_cause!(
        irq_num,
        @TIMER => if !TIMER_HANDLER.is_inited() {
            log::debug!("timer init: {}", TIMER_HANDLER.is_inited());
            TIMER_HANDLER.init_once(handler);
            true
        } else {
            false
        },
        @EXT => crate::irq::register_handler_common(irq_num, handler),
    )
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(irq_num: usize) {
    with_cause!(
        irq_num,
        @TIMER => {
            ticlr::clear_timer_interrupt();
            TIMER_HANDLER();
        },
        @EXT => crate::irq::dispatch_irq_common(0),
    );
}
