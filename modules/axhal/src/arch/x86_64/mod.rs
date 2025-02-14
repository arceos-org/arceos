mod context;
mod gdt;
mod idt;

#[cfg(feature = "uspace")]
mod syscall;

#[cfg(target_os = "none")]
mod trap;

use core::arch::asm;

use memory_addr::{MemoryAddr, PhysAddr, VirtAddr};
use x86::{controlregs, msr, tlb};
use x86_64::instructions::interrupts;

pub use self::context::{ExtendedState, FxsaveArea, TaskContext, TrapFrame};
pub use self::gdt::{GdtStruct, init_gdt, tss_get_rsp0, tss_set_rsp0};
pub use self::idt::{IdtStruct, init_idt};

#[cfg(feature = "uspace")]
pub use self::{context::UspaceContext, syscall::init_syscall};

/// Allows the current CPU to respond to interrupts.
#[inline]
pub fn enable_irqs() {
    #[cfg(not(target_os = "none"))]
    {
        warn!("enable_irqs: not implemented");
    }
    #[cfg(target_os = "none")]
    interrupts::enable()
}

/// Makes the current CPU to ignore interrupts.
#[inline]
pub fn disable_irqs() {
    #[cfg(not(target_os = "none"))]
    {
        warn!("disable_irqs: not implemented");
    }
    #[cfg(target_os = "none")]
    interrupts::disable()
}

/// Returns whether the current CPU is allowed to respond to interrupts.
#[inline]
pub fn irqs_enabled() -> bool {
    interrupts::are_enabled()
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
    pa!(unsafe { controlregs::cr3() } as usize).align_down_4k()
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
        unsafe { controlregs::cr3_write(root_paddr.as_usize() as _) }
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

/// Reads the thread pointer of the current CPU.
///
/// It is used to implement TLS (Thread Local Storage).
#[inline]
pub fn read_thread_pointer() -> usize {
    unsafe { msr::rdmsr(msr::IA32_FS_BASE) as usize }
}

/// Writes the thread pointer of the current CPU.
///
/// It is used to implement TLS (Thread Local Storage).
///
/// # Safety
///
/// This function is unsafe as it changes the CPU states.
#[inline]
pub unsafe fn write_thread_pointer(fs_base: usize) {
    unsafe { msr::wrmsr(msr::IA32_FS_BASE, fs_base as u64) }
}

/// Initializes CPU states on the current CPU.
///
/// In detail, it initializes the GDT, IDT on x86_64 platforms. If the `uspace`
/// feature is enabled, it also initializes relevant model-specific registers
/// to enable the `syscall` instruction.
pub fn cpu_init() {
    init_gdt();
    init_idt();
    #[cfg(feature = "uspace")]
    init_syscall();
}
