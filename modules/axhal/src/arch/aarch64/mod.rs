mod context;

#[cfg(target_os = "none")]
mod trap;

use core::arch::asm;

use aarch64_cpu::registers::{DAIF, TPIDR_EL0, TTBR0_EL1, TTBR1_EL1, VBAR_EL1};
use memory_addr::{PhysAddr, VirtAddr};
use tock_registers::interfaces::{Readable, Writeable};

#[cfg(feature = "uspace")]
pub use self::context::UspaceContext;
pub use self::context::{FpState, TaskContext, TrapFrame};

/// Allows the current CPU to respond to interrupts.
#[inline]
pub fn enable_irqs() {
    unsafe { asm!("msr daifclr, #2") };
}

/// Makes the current CPU to ignore interrupts.
#[inline]
pub fn disable_irqs() {
    unsafe { asm!("msr daifset, #2") };
}

/// Returns whether the current CPU is allowed to respond to interrupts.
#[inline]
pub fn irqs_enabled() -> bool {
    !DAIF.matches_all(DAIF::I::Masked)
}

/// Relaxes the current CPU and waits for interrupts.
///
/// It must be called with interrupts enabled, otherwise it will never return.
#[inline]
pub fn wait_for_irqs() {
    aarch64_cpu::asm::wfi();
}

/// Halt the current CPU.
#[inline]
pub fn halt() {
    disable_irqs();
    aarch64_cpu::asm::wfi(); // should never return
}

/// Reads the register that stores the current page table root.
///
/// Returns the physical address of the page table root.
#[inline]
pub fn read_page_table_root() -> PhysAddr {
    let root = TTBR1_EL1.get();
    pa!(root as usize)
}

/// Reads the `TTBR0_EL1` register.
pub fn read_page_table_root0() -> PhysAddr {
    let root = TTBR0_EL1.get();
    pa!(root as usize)
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
        // kernel space page table use TTBR1 (0xffff_0000_0000_0000..0xffff_ffff_ffff_ffff)
        TTBR1_EL1.set(root_paddr.as_usize() as _);
        flush_tlb(None);
    }
}

/// Writes the `TTBR0_EL1` register.
///
/// # Safety
///
/// This function is unsafe as it changes the virtual memory address space.
pub unsafe fn write_page_table_root0(root_paddr: PhysAddr) {
    TTBR0_EL1.set(root_paddr.as_usize() as _);
    flush_tlb(None);
}

/// Flushes the TLB.
///
/// If `vaddr` is [`None`], flushes the entire TLB. Otherwise, flushes the TLB
/// entry that maps the given virtual address.
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

/// Flushes the entire instruction cache.
#[inline]
pub fn flush_icache_all() {
    unsafe { asm!("ic iallu; dsb sy; isb") };
}

/// Sets the base address of the exception vector (writes `VBAR_EL1`).
#[inline]
pub fn set_exception_vector_base(vbar_el1: usize) {
    VBAR_EL1.set(vbar_el1 as _);
}

/// Flushes the data cache line (64 bytes) at the given virtual address
#[inline]
pub fn flush_dcache_line(vaddr: VirtAddr) {
    unsafe { asm!("dc ivac, {0:x}; dsb sy; isb", in(reg) vaddr.as_usize()) };
}

/// Reads the thread pointer of the current CPU.
///
/// It is used to implement TLS (Thread Local Storage).
#[inline]
pub fn read_thread_pointer() -> usize {
    TPIDR_EL0.get() as usize
}

/// Writes the thread pointer of the current CPU.
///
/// It is used to implement TLS (Thread Local Storage).
///
/// # Safety
///
/// This function is unsafe as it changes the CPU states.
#[inline]
pub unsafe fn write_thread_pointer(tpidr_el0: usize) {
    TPIDR_EL0.set(tpidr_el0 as _)
}

/// Initializes CPU states on the current CPU.
///
/// On AArch64, it sets the exception vector base address (`VBAR_EL1`) and `TTBR0_EL1`.
pub fn cpu_init() {
    unsafe extern "C" {
        fn exception_vector_base();
    }
    set_exception_vector_base(exception_vector_base as usize);
    unsafe { write_page_table_root0(0.into()) }; // disable low address access in EL1
}
