use axconfig::TASK_STACK_SIZE;
use page_table_entry::{GenericPTE, MappingFlags, loongarch64::LA64PTE};

#[unsafe(link_section = ".bss.stack")]
static mut BOOT_STACK: [u8; TASK_STACK_SIZE] = [0; TASK_STACK_SIZE];

#[unsafe(link_section = ".data.boot_page_table")]
static mut BOOT_PT_L0: [LA64PTE; 512] = [LA64PTE::empty(); 512];

#[unsafe(link_section = ".data.boot_page_table")]
static mut BOOT_PT_L1: [LA64PTE; 512] = [LA64PTE::empty(); 512];

unsafe fn init_boot_page_table() {
    unsafe {
        let l1_va = va!(&raw const BOOT_PT_L1 as usize);
        // 0x0000_0000_0000 ~ 0x0080_0000_0000, table
        BOOT_PT_L0[0] = LA64PTE::new_table(crate::mem::virt_to_phys(l1_va));
        // 0x0000_0000..0x4000_0000, VPWXGD, 1G block
        BOOT_PT_L1[0] = LA64PTE::new_page(
            pa!(0),
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
            true,
        );
        // 0x8000_0000..0xc000_0000, VPWXGD, 1G block
        BOOT_PT_L1[2] = LA64PTE::new_page(
            pa!(0x8000_0000),
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
            true,
        );
    }
}

fn enable_fp_simd() {
    // FP/SIMD needs to be enabled early, as the compiler may generate SIMD
    // instructions in the bootstrapping code to speed up the operations
    // like `memset` and `memcpy`.
    #[cfg(feature = "fp-simd")]
    {
        axcpu::asm::enable_fp();
        axcpu::asm::enable_lsx();
    }
}

fn init_mmu() {
    axcpu::init::init_mmu(
        crate::mem::virt_to_phys(va!(&raw const BOOT_PT_L0 as usize)),
        axconfig::plat::PHYS_VIRT_OFFSET,
    );
}

/// The earliest entry point for the primary CPU.
///
/// We can't use bl to jump to higher address, so we use jirl to jump to higher address.
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!("
        ori         $t0, $zero, 0x1     # CSR_DMW1_PLV0
        lu52i.d     $t0, $t0, -2048     # UC, PLV0, 0x8000 xxxx xxxx xxxx
        csrwr       $t0, 0x180          # LOONGARCH_CSR_DMWIN0
        ori         $t0, $zero, 0x11    # CSR_DMW1_MAT | CSR_DMW1_PLV0
        lu52i.d     $t0, $t0, -1792     # CA, PLV0, 0x9000 xxxx xxxx xxxx
        csrwr       $t0, 0x181          # LOONGARCH_CSR_DMWIN1

        # Setup Stack
        la.global   $sp, {boot_stack}
        li.d        $t0, {boot_stack_size}
        add.d       $sp, $sp, $t0       # setup boot stack

        # Init MMU
        bl          {enable_fp_simd}    # enable FP/SIMD instructions
        bl          {init_boot_page_table}
        bl          {init_mmu}          # setup boot page table and enable MMU

        csrrd       $a0, 0x20           # cpuid
        la.global   $t0, {entry}
        jirl        $zero, $t0, 0",
        boot_stack_size = const TASK_STACK_SIZE,
        boot_stack = sym BOOT_STACK,
        enable_fp_simd = sym enable_fp_simd,
        init_boot_page_table = sym init_boot_page_table,
        init_mmu = sym init_mmu,
        entry = sym super::rust_entry,
    )
}

/// The earliest entry point for secondary CPUs.
#[cfg(feature = "smp")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
unsafe extern "C" fn _start_secondary() -> ! {
    core::arch::naked_asm!("
        ori          $t0, $zero, 0x1     # CSR_DMW1_PLV0
        lu52i.d      $t0, $t0, -2048     # UC, PLV0, 0x8000 xxxx xxxx xxxx
        csrwr        $t0, 0x180          # LOONGARCH_CSR_DMWIN0
        ori          $t0, $zero, 0x11    # CSR_DMW1_MAT | CSR_DMW1_PLV0
        lu52i.d      $t0, $t0, -1792     # CA, PLV0, 0x9000 xxxx xxxx xxxx
        csrwr        $t0, 0x181          # LOONGARCH_CSR_DMWIN1
        la.abs       $t0, {sm_boot_stack_top}
        ld.d         $sp, $t0,0          # read boot stack top

        # Init MMU
        bl           {enable_fp_simd}    # enable FP/SIMD instructions
        bl           {init_mmu}          # setup boot page table and enable MMU

        csrrd        $a0, 0x20                  # cpuid
        la.global    $t0, {entry}
        jirl         $zero, $t0, 0",
        sm_boot_stack_top = sym super::mp::SMP_BOOT_STACK_TOP,
        enable_fp_simd = sym enable_fp_simd,
        init_mmu = sym init_mmu,
        entry = sym super::rust_entry_secondary,
    )
}
