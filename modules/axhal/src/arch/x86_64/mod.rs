mod context;

use core::arch::asm;

use memory_addr::{PhysAddr, VirtAddr};
use x86::{bits64::rflags, bits64::rflags::RFlags, controlregs, tlb};

pub use context::{TaskContext, TrapFrame};

#[inline]
pub fn enable_irqs() {
    #[cfg(target_os = "none")]
    unsafe {
        asm!("sti")
    }
}

#[inline]
pub fn disable_irqs() {
    #[cfg(target_os = "none")]
    unsafe {
        asm!("cli")
    }
}

#[inline]
pub fn irqs_enabled() -> bool {
    if cfg!(target_os = "none") {
        !rflags::read().contains(RFlags::FLAGS_IF)
    } else {
        false
    }
}

#[inline]
pub fn wait_for_irqs() {
    if cfg!(target_os = "none") {
        unsafe { asm!("sti; hlt; cli") }
    } else {
        core::hint::spin_loop()
    }
}

#[inline]
pub fn read_page_table_root() -> PhysAddr {
    PhysAddr::from(unsafe { controlregs::cr3() } as usize).align_down_4k()
}

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

#[inline]
pub fn flush_tlb(vaddr: Option<VirtAddr>) {
    if let Some(vaddr) = vaddr {
        unsafe { tlb::flush(vaddr.into()) }
    } else {
        unsafe { tlb::flush_all() }
    }
}
