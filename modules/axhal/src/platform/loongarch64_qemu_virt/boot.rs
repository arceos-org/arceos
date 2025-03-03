use axconfig::TASK_STACK_SIZE;
use loongArch64::register::{pgdh, pgdl};
use page_table_entry::loongarch64::PTEFlags;

#[unsafe(link_section = ".bss.stack")]
static mut BOOT_STACK: [u8; TASK_STACK_SIZE] = [0; TASK_STACK_SIZE];

#[unsafe(link_section = ".data.boot_page_table")]
static mut BOOT_PT_L0: [u64; 512] = [0; 512];

#[unsafe(link_section = ".data.boot_page_table")]
static mut BOOT_PT_L1: [u64; 512] = [0; 512];

unsafe fn init_boot_page_table() {
    const HUGE_FLAGS: PTEFlags = PTEFlags::V
        .union(PTEFlags::D)
        .union(PTEFlags::GH)
        .union(PTEFlags::P)
        .union(PTEFlags::W);
    unsafe {
        let l1_va = va!(&raw const BOOT_PT_L1 as usize);
        // 0x0000_0000_0000 ~ 0x0080_0000_0000, table
        BOOT_PT_L0[0] = crate::mem::virt_to_phys(l1_va).as_usize() as u64;
        // 0x0000_0000..0x4000_0000, VRWX_GAD, 1G block
        BOOT_PT_L1[0] = HUGE_FLAGS.bits();
        // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
        BOOT_PT_L1[2] = 0x8000_0000 | HUGE_FLAGS.bits();
    }
}

unsafe fn init_mmu() {
    crate::arch::init_tlb();

    let paddr = crate::mem::virt_to_phys(va!(&raw const BOOT_PT_L0 as usize));
    pgdh::set_base(paddr.as_usize());
    pgdl::set_base(0);
}

/// The earliest entry point for the primary CPU.
///
/// We can't use bl to jump to higher address, so we use jirl to jump to higher address.
#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
unsafe extern "C" fn _start() -> ! {
    unsafe {
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
            bl          {init_boot_page_table}
            bl          {init_mmu}          # setup boot page table and enabel MMU
            invtlb      0x00, $r0, $r0


            # Enable PG 
            li.w		$t0, 0xb0		# PLV=0, IE=0, PG=1
            csrwr		$t0, 0x0        # LOONGARCH_CSR_CRMD
            li.w		$t0, 0x00		# PLV=0, PIE=0, PWE=0
            csrwr		$t0, 0x1        # LOONGARCH_CSR_PRMD
            li.w		$t0, 0x00		# FPE=0, SXE=0, ASXE=0, BTE=0
            csrwr		$t0, 0x2        # LOONGARCH_CSR_EUEN

            csrrd       $a0, 0x20           # cpuid
            la.global   $t0, {entry}
            jirl        $zero, $t0, 0
            ",
            boot_stack_size = const TASK_STACK_SIZE,
            boot_stack = sym BOOT_STACK,
            init_boot_page_table = sym init_boot_page_table,
            init_mmu = sym init_mmu,
            entry = sym super::rust_entry,
        )
    }
}

/// The earliest entry point for secondary CPUs.
#[cfg(feature = "smp")]
#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
unsafe extern "C" fn _start_secondary() -> ! {
    unsafe {
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
            bl           {init_mmu}          # setup boot page table and enabel MMU
            invtlb       0x00, $r0, $r0

            # Enable PG 
            li.w		$t0, 0xb0		# PLV=0, IE=0, PG=1
            csrwr		$t0, 0x0        # LOONGARCH_CSR_CRMD
            li.w		$t0, 0x00		# PLV=0, PIE=0, PWE=0
            csrwr		$t0, 0x1        # LOONGARCH_CSR_PRMD
            li.w		$t0, 0x00		# FPE=0, SXE=0, ASXE=0, BTE=0
            csrwr		$t0, 0x2        # LOONGARCH_CSR_EUEN

            csrrd        $a0, 0x20                  # cpuid
            la.global    $t0, {entry}
            jirl         $zero, $t0, 0
            ",
            sm_boot_stack_top = sym super::mp::SMP_BOOT_STACK_TOP,
            init_mmu = sym init_mmu,
            entry = sym super::rust_entry_secondary,
        )
    }
}
