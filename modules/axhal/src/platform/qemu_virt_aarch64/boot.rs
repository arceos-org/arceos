use aarch64_cpu::{asm, asm::barrier, registers::*};
use memory_addr::PhysAddr;
use page_table::{aarch64::A64PTE, GenericPTE, MappingFlags as MemFlags};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use axconfig::TASK_STACK_SIZE;

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; TASK_STACK_SIZE] = [0; TASK_STACK_SIZE];

#[link_section = ".data.boot_page_table"]
static mut BOOT_PT_L0: [A64PTE; 512] = [A64PTE::empty(); 512];

#[link_section = ".data.boot_page_table"]
static mut BOOT_PT_L1: [A64PTE; 512] = [A64PTE::empty(); 512];

unsafe fn switch_to_el1() {
    SPSel.write(SPSel::SP::ELx);
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
        SP_EL1.set(BOOT_STACK.as_ptr_range().end as u64);
        ELR_EL2.set(LR.get());
        asm::eret();
    }
}

unsafe fn init_boot_page_table() {
    // 0x0000_0000_0000 ~ 0x0080_0000_0000, table
    BOOT_PT_L0[0] = A64PTE::new_table(PhysAddr::from(BOOT_PT_L1.as_ptr() as usize));
    // 0x0000_0000_0000..0x0000_4000_0000, 1G block, device memory
    BOOT_PT_L1[0] = A64PTE::new_page(
        PhysAddr::from(0),
        MemFlags::READ | MemFlags::WRITE | MemFlags::DEVICE,
        true,
    );
    // 0x0000_4000_0000..0x0000_8000_0000, 1G block, normal memory
    BOOT_PT_L1[1] = A64PTE::new_page(
        PhysAddr::from(0x4000_0000),
        MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
        true,
    );
}

unsafe fn init_mmu() {
    // Device-nGnRE memory
    let attr0 = MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck;
    // Normal memory
    let attr1 = MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc;
    MAIR_EL1.write(attr0 + attr1); // 0xff_04

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
    TCR_EL1.write(TCR_EL1::IPS::Bits_40 + tcr_flags0 + tcr_flags1);
    barrier::isb(barrier::SY);

    // Set both TTBR0 and TTBR1
    let root_paddr = PhysAddr::from(BOOT_PT_L0.as_ptr() as usize).as_usize() as _;
    TTBR0_EL1.set(root_paddr);
    TTBR1_EL1.set(root_paddr);

    // Flush the entire TLB
    crate::arch::flush_tlb(None);

    // Enable the MMU and turn on I-cache and D-cache
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    barrier::isb(barrier::SY);
}

#[naked]
#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn _start() -> ! {
    extern "Rust" {
        fn rust_main();
    }
    // PC = 0x4008_0000
    core::arch::asm!("
        adrp    x8, boot_stack_top
        mov     sp, x8
        bl      {switch_to_el1}
        bl      {init_boot_page_table}
        bl      {init_mmu}

        // set SP to the high address
        ldr     x8, =boot_stack_top
        mov     sp, x8

        // call functions at the high address
        ldr     x8, ={platform_init}
        blr     x8
        ldr     x8, ={rust_main}
        blr     x8
        b      .",
        switch_to_el1 = sym switch_to_el1,
        init_boot_page_table = sym init_boot_page_table,
        init_mmu = sym init_mmu,
        platform_init = sym super::platform_init,
        rust_main = sym rust_main,
        options(noreturn),
    )
}
