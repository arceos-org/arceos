#[macro_use]
mod macros;

mod context;
mod trap;

use memory_addr::{PhysAddr, VirtAddr};
use riscv::asm;
use riscv::register::{satp, sstatus, stvec};

pub use context::{TaskContext, TrapFrame};

#[inline]
pub fn enable_irqs() {
    unsafe { sstatus::set_sie() }
}

#[inline]
pub fn disable_irqs() {
    unsafe { sstatus::clear_sie() }
}

#[inline]
pub fn irqs_enabled() -> bool {
    sstatus::read().sie()
}

#[inline]
pub fn wait_for_irqs() {
    enable_irqs();
    unsafe { riscv::asm::wfi() }
    disable_irqs();
}

#[inline]
pub fn read_page_table_root() -> PhysAddr {
    PhysAddr::from(satp::read().ppn() << 12)
}

/// # Safety
///
/// This function is unsafe as it changes the virtual memory address space.
pub unsafe fn write_page_table_root(root_paddr: PhysAddr) {
    let old_root = read_page_table_root();
    trace!("set page table root: {:#x} => {:#x}", old_root, root_paddr);
    if old_root != root_paddr {
        satp::set(satp::Mode::Sv39, 0, root_paddr.as_usize() >> 12);
        asm::sfence_vma_all();
    }
}

#[inline]
pub fn flush_tlb(vaddr: Option<VirtAddr>) {
    unsafe {
        if let Some(vaddr) = vaddr {
            asm::sfence_vma(0, vaddr.as_usize())
        } else {
            asm::sfence_vma_all();
        }
    }
}

#[inline]
pub fn set_tap_vector_base(stvec: usize) {
    unsafe { stvec::write(stvec, stvec::TrapMode::Direct) }
}

#[inline]
pub fn cpu_id() -> usize {
    // TODO: use `current_cpu().id`
    let mut ret;
    unsafe { core::arch::asm!("mv {}, tp", out(reg) ret) };
    ret
}
