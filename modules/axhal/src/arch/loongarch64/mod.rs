#[macro_use]
mod context;
mod trap;

use core::arch::asm;
use loongarch64::register::{crmd, eentry, pgd};
use memory_addr::{PhysAddr, VirtAddr};

pub use self::context::{TaskContext, TrapFrame};

/// Allows the current CPU to respond to interrupts.
#[inline]
pub fn enable_irqs() {
    crmd::set_ie(true)
}

/// Makes the current CPU to ignore interrupts.
#[inline]
pub fn disable_irqs() {
    crmd::set_ie(false)
}

/// Returns whether the current CPU is allowed to respond to interrupts.
#[inline]
pub fn irqs_enabled() -> bool {
    crmd::read().ie()
}

/// Relaxes the current CPU and waits for interrupts.
///
/// It must be called with interrupts enabled, otherwise it will never return.
#[inline]
pub fn wait_for_irqs() {
    unsafe { loongarch64::asm::idle() }
}

/// Halt the current CPU.
#[inline]
pub fn halt() {
    disable_irqs();
    unsafe { loongarch64::asm::idle() } // should never return
}

/// Reads the register that stores the current page table root.
///
/// Returns the physical address of the page table root.
#[inline]
pub fn read_page_table_root() -> PhysAddr {
    PhysAddr::from(pgd::read().base())
}

/// Writes the register to update the current page table root.
///
/// # Safety
///
/// This function is unsafe as it changes the virtual memory address space.
pub unsafe fn write_page_table_root(root_paddr: PhysAddr) {
    let old_root = read_page_table_root();
    trace!("set page table root: {:#x} => {:#x}", old_root, root_paddr);
    trace!("loongarch64 don't need to set page table");
}

/// Flushes the TLB.
///
/// If `vaddr` is [`None`], flushes the entire TLB. Otherwise, flushes the TLB
/// entry that maps the given virtual address.
#[inline]
pub fn flush_tlb(_vaddr: Option<VirtAddr>) {
    trace!("loongarch64 don't need to flush tlb now");
}

/// Writes Exception Entry Base Address Register (`eentry`).
#[inline]
pub fn set_trap_vector_base(eentry: usize) {
    eentry::set_eentry(eentry);
}

/// Reads the thread pointer of the current CPU.
///
/// It is used to implement TLS (Thread Local Storage).
#[inline]
pub fn read_thread_pointer() -> usize {
    let tp;
    unsafe { asm!("move {}, tp", out(reg) tp) };
    tp
}

/// Writes the thread pointer of the current CPU.
///
/// It is used to implement TLS (Thread Local Storage).
///
/// # Safety
///
/// This function is unsafe as it changes the CPU states.
#[inline]
pub unsafe fn write_thread_pointer(tp: usize) {
    asm!("move tp, {}", in(reg) tp)
}
