mod context;
mod percpu;
mod trap;

pub use self::context::{TaskContext, TrapFrame};
pub use self::percpu::ArchPerCpu;

use core::arch::asm;
use cortex_a::registers::{DAIF, TTBR1_EL1};
use memory_addr::PhysAddr;
use tock_registers::interfaces::{Readable, Writeable};

#[inline]
pub fn enable_irqs() {
    unsafe { asm!("msr daifclr, #2") };
}

#[inline]
pub fn disable_irqs() {
    unsafe { asm!("msr daifset, #2") };
}

#[inline]
pub fn irqs_disabled() -> bool {
    DAIF.matches_all(DAIF::I::Masked)
}

#[inline]
pub fn irqs_enabled() -> bool {
    !irqs_disabled()
}

#[inline]
#[allow(dead_code)]
pub fn wait_for_irqs() {
    cortex_a::asm::wfi();
}

#[inline]
pub fn read_page_table_root() -> PhysAddr {
    let root = TTBR1_EL1.get();
    PhysAddr::from(root as usize)
}

#[inline]
pub unsafe fn write_page_table_root(root_paddr: PhysAddr) {
    let root_paddr = root_paddr.as_usize();
    // kernel space page table use TTBR1 (0xffff_0000_0000_0000..0xffff_ffff_ffff_ffff)
    TTBR1_EL1.set(root_paddr as _);
    flush_tlb_all();
}

#[inline]
pub fn flush_tlb_all() {
    unsafe { asm!("tlbi vmalle1; dsb sy; isb") };
}

#[inline]
pub fn flush_icache_all() {
    unsafe { asm!("ic iallu; dsb sy; isb") };
}
