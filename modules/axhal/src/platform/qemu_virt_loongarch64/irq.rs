use crate::irq::IrqHandler;
use lazy_init::LazyInit;
use loongarch64::register::csr::Register;
use loongarch64::register::tcfg::Tcfg;
pub(super) const CSR_ECFG_VS_SHIFT: usize = 16;
pub(super) const CSR_ECFG_LIE_TI_SHIFT: usize = 11;
pub(super) const TI_VEC: usize = 0x1 << CSR_ECFG_LIE_TI_SHIFT;
/// HWI mask
pub(super) const HWI_VEC: usize = 0x3fc;

pub(super) const SWI_IRQ_NUM: usize = 0;

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 1024;

/// The timer IRQ number
pub const TIMER_IRQ_NUM: usize = 11;

static TIMER_HANDLER: LazyInit<IrqHandler> = LazyInit::new();

macro_rules! with_cause {
    ($cause: expr, @TIMER => $timer_op: expr, @EXT => $ext_op: expr $(,)?) => {
        match $cause {
            TIMER_IRQ_NUM => $timer_op,
            S_EXT => $ext_op,
            _ => panic!("invalid trap cause: {:#x}", $cause),
        }
    };
}
/// Enables or disables the given IRQ.
pub fn set_enable(vector: usize, enabled: bool) {
    if vector == 11 {
        if enabled {
            Tcfg::read()
                .set_enable(true)
                .set_initval(800000000 as usize)
                .set_loop(false)
                .write();
        }
    }
}

/// Registers an IRQ handler for the given IRQ.
///
/// It also enables the IRQ if the registration succeeds. It returns `false` if
/// the registration failed.
///
pub fn register_handler(vector: usize, handler: IrqHandler) -> bool {
    crate::irq::register_handler_common(vector, handler)
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(vector: usize) {
    crate::irq::dispatch_irq_common(vector);
}

pub(super) fn init_primary() {
    // enable soft interrupts, timer interrupts, and external interrupts
    // disable_irqs();
}
