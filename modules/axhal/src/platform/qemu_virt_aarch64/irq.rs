use crate::irq::IrqHandler;

pub const MAX_IRQ_COUNT: usize = 1024;

pub fn set_enable(_irq_num: usize, _enabled: bool) {
    // TODO: set enable in GIC
}

pub fn register_handler(irq_num: usize, handler: IrqHandler) -> bool {
    crate::irq::register_handler_common(irq_num, handler)
}

/// Platform-dependent IRQ handler
pub(crate) fn platform_handle_irq(_unused: usize) {
    // TODO: get IRQ number from GIC
    crate::irq::dispatch_irq(0);
}

pub(super) fn init() {}
