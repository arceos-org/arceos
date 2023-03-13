mod context;
pub(crate) mod trap;

use core::arch::asm;

use aarch64_cpu::registers::{DAIF, MPIDR_EL1, TTBR1_EL1, VBAR_EL1};
use memory_addr::{PhysAddr, VirtAddr};
use tock_registers::interfaces::{Readable, Writeable};

pub use self::context::{TaskContext, TrapFrame};

#[inline]
pub fn enable_irqs() {
    unsafe { asm!("msr daifclr, #2") };
}

#[inline]
pub fn disable_irqs() {
    unsafe { asm!("msr daifset, #2") };
}

#[inline]
pub fn irqs_enabled() -> bool {
    !DAIF.matches_all(DAIF::I::Masked)
}

#[inline]
pub fn wait_for_irqs() {
    aarch64_cpu::asm::wfi();
}

#[inline]
pub fn read_page_table_root() -> PhysAddr {
    let root = TTBR1_EL1.get();
    PhysAddr::from(root as usize)
}

/// # Safety
///
/// This function is unsafe as it changes the virtual memory address space.
pub unsafe fn write_page_table_root(root_paddr: PhysAddr) {
    let old_root = read_page_table_root();
    trace!("set page table root: {:#x} => {:#x}", old_root, root_paddr);
    if old_root != root_paddr {
        // kernel space page table use TTBR1 (0xffff_0000_0000_0000..0xffff_ffff_ffff_ffff)
        TTBR1_EL1.set(root_paddr.as_usize() as _);
        flush_tlb(None);
    }
}

#[inline]
pub fn flush_tlb(vaddr: Option<VirtAddr>) {
    unsafe {
        if let Some(vaddr) = vaddr {
            asm!("tlbi vaae1is, {}; dsb sy; isb", in(reg) vaddr.as_usize())
        } else {
            // flush the entire TLB
            asm!("tlbi vmalle1; dsb sy; isb")
        }
    }
}

#[inline]
pub fn flush_icache_all() {
    unsafe { asm!("ic iallu; dsb sy; isb") };
}

#[inline]
pub fn set_exception_vector_base(vbar_el1: usize) {
    VBAR_EL1.set(vbar_el1 as _);
}

#[inline]
pub fn cpu_id() -> usize {
    // TODO: use `current_cpu().id`
    (MPIDR_EL1.get() & 0xff_ff_ff) as usize
}
