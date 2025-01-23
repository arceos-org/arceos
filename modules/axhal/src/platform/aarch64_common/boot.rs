use aarch64_cpu::{asm, asm::barrier, registers::*};
use core::ptr::addr_of_mut;
use page_table_entry::aarch64::{A64PTE, MemAttr};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use axconfig::{TASK_STACK_SIZE, plat::PHYS_VIRT_OFFSET};

#[unsafe(link_section = ".bss.stack")]
static mut BOOT_STACK: [u8; TASK_STACK_SIZE] = [0; TASK_STACK_SIZE];

#[unsafe(link_section = ".data.boot_page_table")]
static mut BOOT_PT_L0: [A64PTE; 512] = [A64PTE::empty(); 512];

#[unsafe(link_section = ".data.boot_page_table")]
static mut BOOT_PT_L1: [A64PTE; 512] = [A64PTE::empty(); 512];

const FLAG_LE: usize = 0b0;
const FLAG_PAGE_SIZE_4K: usize = 0b10;
const FLAG_ANY_MEM: usize = 0b1000;

unsafe fn switch_to_el1() {
    SPSel.write(SPSel::SP::ELx);
    SP_EL0.set(0);
    let current_el = CurrentEL.read(CurrentEL::EL);
    if current_el >= 2 {
        if current_el == 3 {
            // Set EL2 to 64bit and enable the HVC instruction.
            SCR_EL3.write(
                SCR_EL3::NS::NonSecure + SCR_EL3::HCE::HvcEnabled + SCR_EL3::RW::NextELIsAarch64,
            );
            // Set the return address and exception level.
            SPSR_EL3.write(
                SPSR_EL3::M::EL1h
                    + SPSR_EL3::D::Masked
                    + SPSR_EL3::A::Masked
                    + SPSR_EL3::I::Masked
                    + SPSR_EL3::F::Masked,
            );
            ELR_EL3.set(LR.get());
        }
        // Disable EL1 timer traps and the timer offset.
        CNTHCTL_EL2.modify(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
        CNTVOFF_EL2.set(0);
        // Set EL1 to 64bit.
        HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
        // Set the return address and exception level.
        SPSR_EL2.write(
            SPSR_EL2::M::EL1h
                + SPSR_EL2::D::Masked
                + SPSR_EL2::A::Masked
                + SPSR_EL2::I::Masked
                + SPSR_EL2::F::Masked,
        );
        unsafe {
            core::arch::asm!(
                "
                mov     x8, sp
                msr     sp_el1, x8"
            )
        };
        ELR_EL2.set(LR.get());
        asm::eret();
    }
}

unsafe fn init_mmu() {
    MAIR_EL1.set(MemAttr::MAIR_VALUE);

    // Enable TTBR0 and TTBR1 walks, page size = 4K, vaddr size = 48 bits, paddr size = 40 bits.
    let tcr_flags0 = TCR_EL1::EPD0::EnableTTBR0Walks
        + TCR_EL1::TG0::KiB_4
        + TCR_EL1::SH0::Inner
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T0SZ.val(16);
    let tcr_flags1 = TCR_EL1::EPD1::EnableTTBR1Walks
        + TCR_EL1::TG1::KiB_4
        + TCR_EL1::SH1::Inner
        + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T1SZ.val(16);
    TCR_EL1.write(TCR_EL1::IPS::Bits_48 + tcr_flags0 + tcr_flags1);
    barrier::isb(barrier::SY);

    // Set both TTBR0 and TTBR1
    let root_paddr = pa!(&raw const BOOT_PT_L0 as usize).as_usize() as _;
    TTBR0_EL1.set(root_paddr);
    TTBR1_EL1.set(root_paddr);

    // Flush the entire TLB
    crate::arch::flush_tlb(None);

    // Enable the MMU and turn on I-cache and D-cache
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    barrier::isb(barrier::SY);
}

unsafe fn enable_fp() {
    if cfg!(feature = "fp_simd") {
        CPACR_EL1.write(CPACR_EL1::FPEN::TrapNothing);
        barrier::isb(barrier::SY);
    }
}

unsafe fn init_boot_page_table() {
    crate::platform::mem::init_boot_page_table(addr_of_mut!(BOOT_PT_L0), addr_of_mut!(BOOT_PT_L1));
}

/// Kernel entry point with Linux image header.
///
/// Some bootloaders require this header to be present at the beginning of the
/// kernel image.
///
/// Documentation: <https://docs.kernel.org/arch/arm64/booting.html>
#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
unsafe extern "C" fn _start() -> ! {
    unsafe {
        // PC = bootloader load address
        // X0 = dtb
        core::arch::naked_asm!("
            add     x13, x18, #0x16     // 'MZ' magic
            b       {entry}             // Branch to kernel start, magic

            .quad   0                   // Image load offset from start of RAM, little-endian
            .quad   _ekernel - _start   // Effective size of kernel image, little-endian
            .quad   {flags}             // Kernel flags, little-endian
            .quad   0                   // reserved
            .quad   0                   // reserved
            .quad   0                   // reserved
            .ascii  \"ARM\\x64\"        // Magic number
            .long   0                   // reserved (used for PE COFF offset)",
            flags = const FLAG_LE | FLAG_PAGE_SIZE_4K | FLAG_ANY_MEM,
            entry = sym _start_primary,
        )
    }
}

/// The earliest entry point for the primary CPU.
#[naked]
#[unsafe(link_section = ".text.boot")]
unsafe extern "C" fn _start_primary() -> ! {
    unsafe {
        // X0 = dtb
        core::arch::naked_asm!("
            mrs     x19, mpidr_el1
            and     x19, x19, #0xffffff     // get current CPU id
            mov     x20, x0                 // save DTB pointer

            adrp    x8, {boot_stack}        // setup boot stack
            add     x8, x8, {boot_stack_size}
            mov     sp, x8

            bl      {switch_to_el1}         // switch to EL1
            bl      {enable_fp}             // enable fp/neon
            bl      {init_boot_page_table}
            bl      {init_mmu}              // setup MMU

            mov     x8, {phys_virt_offset}  // set SP to the high address
            add     sp, sp, x8

            mov     x0, x19                 // call rust_entry(cpu_id, dtb)
            mov     x1, x20
            ldr     x8, ={entry}
            blr     x8
            b      .",
            switch_to_el1 = sym switch_to_el1,
            init_boot_page_table = sym init_boot_page_table,
            init_mmu = sym init_mmu,
            enable_fp = sym enable_fp,
            boot_stack = sym BOOT_STACK,
            boot_stack_size = const TASK_STACK_SIZE,
            phys_virt_offset = const PHYS_VIRT_OFFSET,
            entry = sym crate::platform::rust_entry,
        )
    }
}

/// The earliest entry point for the secondary CPUs.
#[cfg(feature = "smp")]
#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
unsafe extern "C" fn _start_secondary() -> ! {
    unsafe {
        core::arch::naked_asm!("
            mrs     x19, mpidr_el1
            and     x19, x19, #0xffffff     // get current CPU id

            mov     sp, x0
            bl      {switch_to_el1}
            bl      {init_mmu}
            bl      {enable_fp}

            mov     x8, {phys_virt_offset}  // set SP to the high address
            add     sp, sp, x8

            mov     x0, x19                 // call rust_entry_secondary(cpu_id)
            ldr     x8, ={entry}
            blr     x8
            b      .",
            switch_to_el1 = sym switch_to_el1,
            init_mmu = sym init_mmu,
            enable_fp = sym enable_fp,
            phys_virt_offset = const PHYS_VIRT_OFFSET,
            entry = sym crate::platform::rust_entry_secondary,
        )
    }
}
