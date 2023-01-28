use core::arch::asm;

use memory_addr::{PhysAddr, VirtAddr, PAGE_SIZE_4K};
use x86::{bits64::rflags, bits64::rflags::RFlags, controlregs, tlb};

#[inline]
pub fn enable_irqs() {
    unsafe { asm!("sti") };
}

#[inline]
pub fn disable_irqs() {
    unsafe { asm!("cli") };
}

#[inline]
pub fn irqs_enabled() -> bool {
    !rflags::read().contains(RFlags::FLAGS_IF)
}

#[inline]
pub fn read_page_table_root() -> PhysAddr {
    PhysAddr::from(unsafe { controlregs::cr3() } as usize).align_down(PAGE_SIZE_4K)
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
