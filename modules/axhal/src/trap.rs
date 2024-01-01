//! Trap handling.
use super::arch::TrapFrame;
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
    fn handle_irq(irq_num: usize);
    // more e.g.: handle_page_fault();

    // 需要分离用户态使用
    #[cfg(feature = "monolithic")]
    fn handle_syscall(syscall_id: usize, args: [usize; 6]) -> isize;

    #[cfg(feature = "monolithic")]
    fn handle_page_fault(addr: VirtAddr, flags: MappingFlags, tf: &mut TrapFrame);

    /// 处理当前进程的信号
    #[cfg(feature = "signal")]
    fn handle_signal();
}

/// Call the external IRQ handler.
#[allow(dead_code)]
pub(crate) fn handle_irq_extern(irq_num: usize) {
    call_interface!(TrapHandler::handle_irq, irq_num);
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
pub(crate) fn handle_page_fault(addr: VirtAddr, flags: MappingFlags, tf: &mut TrapFrame) {
    call_interface!(TrapHandler::handle_page_fault, addr, flags, tf);
}

/// 信号处理函数
#[allow(dead_code)]
#[cfg(feature = "signal")]
pub(crate) fn handle_signal() {
    call_interface!(TrapHandler::handle_signal);
}
