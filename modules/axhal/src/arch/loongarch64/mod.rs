#[macro_use]
mod context;
mod trap;

use core::arch::asm;
use loongArch64::register::{
    crmd, ecfg, eentry, euen, pgd, pgdh, pgdl, pwch, pwcl, stlbps, tlbidx, tlbrehi, tlbrentry,
};
use memory_addr::{PhysAddr, VirtAddr};

pub use self::context::{TaskContext, TrapFrame};

#[cfg(feature = "uspace")]
pub use self::context::UspaceContext;

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
    unsafe { loongArch64::asm::idle() }
}

/// Halt the current CPU.
#[inline]
pub fn halt() {
    unsafe { loongArch64::asm::idle() }
    disable_irqs();
}

/// Reads the register that stores the current page table root.
///
/// Returns the physical address of the page table root.
#[inline]
pub fn read_page_table_root() -> PhysAddr {
    PhysAddr::from(pgd::read().base())
}

/// Writes the `pgdl` register.
///
/// # Safety
///
/// This function is unsafe as it changes the virtual memory address space.
pub unsafe fn write_page_table_root0(root_paddr: PhysAddr) {
    pgdl::set_base(root_paddr.as_usize() as _);
    flush_tlb(None);
}

/// Writes the register to update the current page table root.
///
/// # Safety
///
/// This function is unsafe as it changes the virtual memory address space.
/// NOTE: Compiler optimize inline on release mode, kernel raise error about
/// page table. So we prohibit inline operation.
#[inline(never)]
pub unsafe fn write_page_table_root(root_paddr: PhysAddr) {
    let old_root = read_page_table_root();
    trace!("set page table root: {:#x} => {:#x}", old_root, root_paddr);

    pgdh::set_base(root_paddr.as_usize());
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
            // <https://loongson.github.io/LoongArch-Documentation/LoongArch-Vol1-EN.html#_dbar>
            //
            // Only after all previous load/store access operations are completely
            // executed, the DBAR 0 instruction can be executed; and only after the
            // execution of DBAR 0 is completed, all subsequent load/store access
            // operations can be executed.
            //
            // <https://loongson.github.io/LoongArch-Documentation/LoongArch-Vol1-EN.html#_invtlb>
            //
            // formats: invtlb op, asid, addr
            //
            // op 0x5: Clear all page table entries with G=0 and ASID equal to the
            // register specified ASID, and VA equal to the register specified VA.
            //
            // When the operation indicated by op does not require an ASID, the
            // general register rj should be set to r0.
            asm!("dbar 0; invtlb 0x05, $r0, {reg}", reg = in(reg) vaddr.as_usize());
        } else {
            // op 0x0: Clear all page table entries
            asm!("dbar 0; invtlb 0x00, $r0, $r0");
        }
    }
}

/// Writes Exception Entry Base Address Register (`eentry`).
///
/// - ecfg: <https://loongson.github.io/LoongArch-Documentation/LoongArch-Vol1-EN.html#exception-configuration>
/// - eentry: <https://loongson.github.io/LoongArch-Documentation/LoongArch-Vol1-EN.html#exception-entry-base-address>
#[inline]
pub fn set_trap_vector_base(eentry: usize) {
    ecfg::set_vs(0);
    eentry::set_eentry(eentry);
}

/// Writes TLB Refill Entry Address Register (`tlbrentry`)
///
/// tlbrentry: <https://loongson.github.io/LoongArch-Documentation/LoongArch-Vol1-EN.html#tlb-refill-exception-entry-base-address>
#[inline]
pub fn set_tlb_refill(tlbrentry: usize) {
    tlbrentry::set_tlbrentry(tlbrentry);
}

/// Init the TLB configuration and set tlb refill handler.
pub fn init_tlb() {
    // Page Size 4KB
    const PS_4K: usize = 0x0c;
    tlbidx::set_ps(PS_4K);
    stlbps::set_ps(PS_4K);
    tlbrehi::set_ps(PS_4K);

    // Set Page table entry width
    pwcl::set_pte_width(8);
    // Set Page table width and offset
    pwcl::set_ptbase(12);
    pwcl::set_ptwidth(9);
    pwcl::set_dir1_base(21);
    pwcl::set_dir1_width(9);
    pwcl::set_dir2_base(30);
    pwcl::set_dir2_width(9);
    pwch::set_dir3_base(39);
    pwch::set_dir3_width(9);

    unsafe extern "C" {
        fn handle_tlb_refill();
    }
    let paddr = crate::mem::virt_to_phys(va!(handle_tlb_refill as usize));
    crate::arch::set_tlb_refill(paddr.as_usize());
}

/// Reads the thread pointer of the current CPU.
///
/// It is used to implement TLS (Thread Local Storage).
#[inline]
pub fn read_thread_pointer() -> usize {
    let tp;
    unsafe { asm!("move {}, $tp", out(reg) tp) };
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
    unsafe { asm!("move $tp, {}", in(reg) tp) }
}

/// Initializes CPU states on the current CPU.
pub fn cpu_init() {
    // Enable floating point
    euen::set_fpe(true);

    unsafe extern "C" {
        fn trap_vector_base();
    }
    set_trap_vector_base(trap_vector_base as usize);
}
