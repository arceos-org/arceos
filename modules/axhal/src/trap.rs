use crate_interface::{call_interface, def_interface};

#[def_interface]
pub trait TrapHandler {
    fn handle_irq(irq_num: usize);

    #[cfg(feature = "user")]
    fn handle_syscall(id: usize, params: [usize; 6]) -> isize;
    // more e.g.: handle_page_fault();
}

/// Call the external IRQ handler.
#[allow(dead_code)]
pub(crate) fn handle_irq_extern(irq_num: usize) {
    call_interface!(TrapHandler::handle_irq, irq_num);
}

/// Call the external syscall handler.
#[cfg(feature = "user")]
#[allow(dead_code)]
pub(crate) fn handle_syscall_extern(syscall_num: usize, param: [usize; 6]) -> isize {
    call_interface!(TrapHandler::handle_syscall, syscall_num, param)
}
