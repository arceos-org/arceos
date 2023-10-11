use crate::irq::IrqHandler;
use lazy_init::LazyInit;
use loongarch64::register::ecfg;
use loongarch64::register::ecfg::LineBasedInterrupt;
use loongarch64::register::ticlr;
/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 1024;

/// The timer IRQ number
pub const TIMER_IRQ_NUM: usize = 11;

/// HW0
pub const EXT_IRQ_NUM: usize = 2;

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
pub fn set_enable(vector: usize, _enabled: bool) {
    if vector == 11 {}
}

/// Registers an IRQ handler for the given IRQ.
///
/// It also enables the IRQ if the registration succeeds. It returns `false` if
/// the registration failed.
pub fn register_handler(vector: usize, handler: IrqHandler) -> bool {
    with_cause!(
        vector,
        @TIMER => if !TIMER_HANDLER.is_init() {
            TIMER_HANDLER.init_by(handler);
            true
        } else {
            false
        },
        @EXT => crate::irq::register_handler_common(vector, handler),
    )
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(vector: usize) {
    with_cause!(
        vector,
        @TIMER => {
            ticlr::clear_timer_interrupt();
            TIMER_HANDLER();
        },
        @EXT => crate::irq::dispatch_irq_common(0),
    );
}

pub(super) fn init_percpu() {
    // enable soft interrupts, timer interrupts, and external interrupts
    let inter = LineBasedInterrupt::TIMER
        | LineBasedInterrupt::SWI0
        | LineBasedInterrupt::SWI1
        | LineBasedInterrupt::HWI0;
    ecfg::set_lie(inter);
}
