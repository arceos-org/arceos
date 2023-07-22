//! Trap handling.

use crate_interface::{call_interface, def_interface};

use crate::arch::TrapFrame;
use memory_addr::VirtAddr;
use page_table::MappingFlags;

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

    #[cfg(feature = "paging")]
    fn handle_page_fault(addr: VirtAddr, flags: MappingFlags, tf: &mut TrapFrame);

    #[cfg(feature = "paging")]
    fn handle_access_fault(addr: VirtAddr, flags: MappingFlags);

    /// 处理当前进程的信号
    #[cfg(feature = "signal")]
    fn handle_signal();

    /// 为了lmbench特判，即在出现未能处理的情况，不panic，而是退出当前进程
    #[cfg(feature = "monolithic")]
    fn exit();
}
/// Call the external IRQ handler.
#[allow(dead_code)]
pub(crate) fn handle_irq_extern(irq_num: usize) {
    call_interface!(TrapHandler::handle_irq, irq_num);
}

/// Call the syscall handler
#[allow(dead_code)]
#[cfg(feature = "monolithic")]
/// 分割token流
pub(crate) fn handle_syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    call_interface!(TrapHandler::handle_syscall, syscall_id, args)
}

#[allow(dead_code)]
#[cfg(feature = "paging")]
pub(crate) fn handle_page_fault(addr: VirtAddr, flags: MappingFlags, tf: &mut TrapFrame) {
    call_interface!(TrapHandler::handle_page_fault, addr, flags, tf);
}

#[allow(dead_code)]
#[cfg(feature = "paging")]
pub(crate) fn handle_access_fault(addr: VirtAddr, flags: MappingFlags) {
    call_interface!(TrapHandler::handle_access_fault, addr, flags)
}

/// 信号处理函数
#[allow(dead_code)]
#[cfg(feature = "signal")]
pub(crate) fn handle_signal() {
    call_interface!(TrapHandler::handle_signal);
}

/// 为了lmbench特判，即在出现未能处理的情况，不panic，而是退出当前进程
#[allow(dead_code)]
#[cfg(feature = "monolithic")]
pub fn exit() {
    call_interface!(TrapHandler::exit);
}
