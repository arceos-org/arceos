mod context;

use core::arch::asm;

use memory_addr::{PhysAddr, VirtAddr};
use x86::{bits64::rflags, bits64::rflags::RFlags, controlregs, tlb};

pub use context::{ExtendedState, FxsaveArea, TaskContext, TrapFrame};

/// Allows the current CPU to respond to interrupts.
#[inline]
pub fn enable_irqs() {
    #[cfg(target_os = "none")]
    unsafe {
        asm!("sti")
    }
}

/// Makes the current CPU to ignore interrupts.
#[inline]
pub fn disable_irqs() {
    #[cfg(target_os = "none")]
    unsafe {
        asm!("cli")
    }
}

/// Returns whether the current CPU is allowed to respond to interrupts.
#[inline]
pub fn irqs_enabled() -> bool {
    if cfg!(target_os = "none") {
        !rflags::read().contains(RFlags::FLAGS_IF)
    } else {
        false
    }
}

/// Relaxes the current CPU and waits for interrupts.
///
/// It must be called with interrupts enabled, otherwise it will never return.
#[inline]
pub fn wait_for_irqs() {
    if cfg!(target_os = "none") {
        unsafe { asm!("hlt") }
    } else {
        core::hint::spin_loop()
    }
}

/// Halt the current CPU.
#[inline]
pub fn halt() {
    disable_irqs();
    wait_for_irqs(); // should never return
}

/// Reads the register that stores the current page table root.
///
/// Returns the physical address of the page table root.
#[inline]
pub fn read_page_table_root() -> PhysAddr {
    PhysAddr::from(unsafe { controlregs::cr3() } as usize).align_down_4k()
}

/// Writes the register to update the current page table root.
///
/// # Safety
///
/// This function is unsafe as it changes the virtual memory address space.
pub unsafe fn write_page_table_root(root_paddr: PhysAddr) {
    let old_root = read_page_table_root();
    trace!("set page table root: {:#x} => {:#x}", old_root, root_paddr);
    if old_root != root_paddr {
        controlregs::cr3_write(root_paddr.as_usize() as _)
    }
}

/// Flushes the TLB.
///
/// If `vaddr` is [`None`], flushes the entire TLB. Otherwise, flushes the TLB
/// entry that maps the given virtual address.
#[inline]
pub fn flush_tlb(vaddr: Option<VirtAddr>) {
    if let Some(vaddr) = vaddr {
        unsafe { tlb::flush(vaddr.into()) }
    } else {
        unsafe { tlb::flush_all() }
    }
}
