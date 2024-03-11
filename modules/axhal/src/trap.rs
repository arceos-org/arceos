//! Trap handling.

use crate_interface::{call_interface, def_interface};
use memory_addr::VirtAddr;
use page_table_entry::MappingFlags;
/// Trap handler interface.
///
/// This trait is defined with the [`#[def_interface]`][1] attribute. Users
/// should implement it with [`#[impl_interface]`][2] in any other crate.
///
/// [1]: crate_interface::def_interface
/// [2]: crate_interface::impl_interface
#[def_interface]
pub trait TrapHandler {
    /// Handles interrupt requests for the given IRQ number.
    fn handle_irq(irq_num: usize, from_user: bool);
    // more e.g.: handle_page_fault();

    #[cfg(feature = "monolithic")]
    /// Handles system calls for the given syscall ID and arguments.
    fn handle_syscall(syscall_id: usize, args: [usize; 6]) -> isize;

    #[cfg(feature = "monolithic")]
    /// Handles page faults.
    fn handle_page_fault(addr: VirtAddr, flags: MappingFlags);

    #[cfg(feature = "signal")]
    /// Handles signals.
    fn handle_signal();
}

/// Call the external IRQ handler.
#[allow(dead_code)]
pub(crate) fn handle_irq_extern(irq_num: usize, from_user: bool) {
    call_interface!(TrapHandler::handle_irq, irq_num, from_user);
}

#[allow(dead_code)]
#[cfg(feature = "monolithic")]
/// 分割token流
#[no_mangle]
pub(crate) fn handle_syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    call_interface!(TrapHandler::handle_syscall, syscall_id, args)
}

#[allow(dead_code)]
#[cfg(feature = "monolithic")]
pub(crate) fn handle_page_fault(addr: VirtAddr, flags: MappingFlags) {
    call_interface!(TrapHandler::handle_page_fault, addr, flags);
}

/// 信号处理函数
#[allow(dead_code)]
#[cfg(feature = "signal")]
pub(crate) fn handle_signal() {
    call_interface!(TrapHandler::handle_signal);
}
