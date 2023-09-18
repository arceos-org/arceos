#[macro_use]
//mod macros;

mod context;
mod trap;

use bit_field::BitField;
use core::arch::asm;
use loongarch64::register::{crmd::Crmd, csr::Register, eentry::Eentry};

use loongarch64::tlb::{Pgd, Pgdl, StlbPs, TLBREntry, TlbREhi};
use memory_addr::{PhysAddr, VirtAddr};

pub use self::context::{TaskContext, TrapFrame};

/// Allows the current CPU to respond to interrupts.
#[inline]
pub fn enable_irqs() {
    Crmd::read().set_ie(true).write()
}

/// Makes the current CPU to ignore interrupts.
#[inline]
pub fn disable_irqs() {
    Crmd::read().set_ie(false).write()
}

/// Returns whether the current CPU is allowed to respond to interrupts.
#[inline]
pub fn irqs_enabled() -> bool {
    Crmd::read().get_ie()
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
    unsafe { loongarch64::asm::idle() } // should never return
    disable_irqs();
}

/// Reads the register that stores the current page table root.
///
/// Returns the physical address of the page table root.
#[inline]
pub fn read_page_table_root() -> PhysAddr {
    PhysAddr::from(Pgd::read().pgd)
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
        Pgdl::read().set_val(root_paddr.into()).write(); //设置新的页基址
                                                         // Pgdh::read().set_val(root_paddr.into()).write(); //设置新的页基址
    }
}

/// Flushes the TLB.
///
/// If `vaddr` is [`None`], flushes the entire TLB. Otherwise, flushes the TLB
/// entry that maps the given virtual address.
#[inline]
pub fn flush_tlb(_vaddr: Option<VirtAddr>) {
    unsafe {
        /*
        if let Some(vaddr) = vaddr {
            asm!("invtlb 0x6,$r0,{}", in(reg) vaddr.as_usize());
        } else {
            asm!("invtlb 0,$r0,$r0");
        }*/
        asm!("tlbflush");
    }
}

/// Writes Exception Entry Base Address Register (`eentry`).
#[inline]
pub fn set_trap_vector_base(eentry: usize) {
    // TODO!(记录状态并恢复)
    Crmd::read().set_ie(false).write(); //关闭全局中断
    Eentry::read().set_eentry(eentry).write(); //设置例外入口
    Crmd::read().set_ie(true).write(); //开启全局中断
}

core::arch::global_asm!(include_str!("tlb.S"));

extern "C" {
    fn tlb_refill_handler();
}

/// Writes TLB Refill Exception Entry Base Address (`tlbrentry`).
#[inline]
pub fn init_tlb() {
    StlbPs::read().set_page_size(0xc).write(); //设置TLB的页面大小为4KiB
    TlbREhi::read().set_page_size(0xc).write(); //设置TLB的页面大小为4KiB
    set_tlb_handler(tlb_refill_handler as usize);
}

/// Writes TLB Refill Exception Entry Base Address (`tlbrentry`).
#[inline]
pub fn set_tlb_handler(tlb_refill_entry: usize) {
    TLBREntry::read()
        .set_val((tlb_refill_entry).get_bits(0..32))
        .write();
}
