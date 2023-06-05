//! Trap handling.

use crate_interface::{call_interface, def_interface};

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

    /// Handles syscalls
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


#[cfg(feature = "user-paging")]
use crate::arch::TrapFrame;
/// Task Infomation Interface
///
/// Call the external task manager to obtain infomation
/// 
/// This trait is defined with the [`#[def_interface]`][1] attribute. Users
/// should implement it with [`#[impl_interface]`][2] in any other crate.
/// 
/// [1]: crate_interface::def_interface
/// [2]: crate_interface::impl_interface
#[cfg(feature = "user-paging")]
#[def_interface]
pub trait CurrentTask {
    /// Gets `TrapFrame` of current task
    fn current_trap_frame() -> *mut TrapFrame;
    /// Gets root of page table of current task
    fn current_satp() -> usize;
    /// Get virtual address of the trap frame of current task
    fn current_trap_frame_virt_addr() -> usize;
}

#[allow(unused)]
#[cfg(feature = "user-paging")]
pub(crate) fn get_current_trap_frame() -> *mut TrapFrame {
    call_interface!(CurrentTask::current_trap_frame)
}
#[allow(unused)]
#[cfg(feature = "user-paging")]
pub(crate) fn get_current_satp() -> usize {
    call_interface!(CurrentTask::current_satp)
}
#[allow(unused)]
#[cfg(feature = "user-paging")]
pub(crate) fn get_current_trap_frame_virt_addr() -> usize {
    call_interface!(CurrentTask::current_trap_frame_virt_addr)
}
