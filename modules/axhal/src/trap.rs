use crate_interface::{call_interface, def_interface};

#[def_interface]
pub trait TrapHandler {
    fn task_try_preempt();
    // more e.g.: handle_page_fault();
}

#[allow(dead_code)]
pub(crate) fn task_try_preempt() {
    call_interface!(TrapHandler::task_try_preempt);
}
