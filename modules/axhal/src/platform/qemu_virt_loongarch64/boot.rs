use axconfig::TASK_STACK_SIZE;

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; TASK_STACK_SIZE] = [0; TASK_STACK_SIZE];
#[allow(unused)]
pub static mut SMP_BOOT_STACK_TOP: usize = 0;

unsafe fn init_mmu() {
    use loongarch64::register::csr::Register;
    use loongarch64::tlb::Pwch;
    use loongarch64::tlb::Pwcl;

    Pwcl::read()
        .set_ptbase(12) //页表起始位置
        .set_ptwidth(9) //页表宽度为9位
        .set_dir1_base(21) //第一级页目录表起始位置
        .set_dir1_width(9) //第一级页目录表宽度为9位
        .set_dir2_base(30) //第二级页目录表起始位置
        .set_dir2_width(9) //第二级页目录表宽度为9位
        .write();
    Pwch::read()
        .set_dir3_base(39) //第三级页目录表
        .set_dir3_width(8) //第三级页目录表宽度为9位
        //.set_dir4_base(48) //第四级页目录表
        .set_dir4_width(0) //第四级页目录表
        .write();
}

/// The earliest entry point for the primary CPU.
///
/// We can't use bl to jump to higher address, so we use jirl to jump to higher address.
#[naked]
#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn _start() -> ! {
    core::arch::asm!("
            ori         $t0, $zero, 0x1     # CSR_DMW1_PLV0
            lu52i.d     $t0, $t0, -2048     # UC, PLV0, 0x8000 xxxx xxxx xxxx
            csrwr       $t0, 0x180          # LOONGARCH_CSR_DMWIN0
            ori         $t0, $zero, 0x11    # CSR_DMW1_MAT | CSR_DMW1_PLV0
            lu52i.d     $t0, $t0, -1792     # CA, PLV0, 0x9000 xxxx xxxx xxxx
            csrwr       $t0, 0x181          # LOONGARCH_CSR_DMWIN1

            bl {init_mmu}

            # Enable PG 
            li.w		$t0, 0xb0		# PLV=0, IE=0, PG=1
            csrwr		$t0, 0x0        # LOONGARCH_CSR_CRMD
            li.w		$t0, 0x00		# PLV=0, PIE=0, PWE=0
            csrwr		$t0, 0x1        # LOONGARCH_CSR_PRMD
            li.w		$t0, 0x00		# FPE=0, SXE=0, ASXE=0, BTE=0
            csrwr		$t0, 0x2        # LOONGARCH_CSR_EUEN
            
            bl {init_tlb}

            la.global   $sp, {boot_stack}
            li.d        $t0, {boot_stack_size}
            add.d       $sp, $sp, $t0       # setup boot stack
            csrrd       $a0, 0x20           # cpuid
            la.global $t0, {entry}
            jirl $zero,$t0,0
            ",
        boot_stack_size = const TASK_STACK_SIZE,
        boot_stack = sym BOOT_STACK,
        init_mmu = sym init_mmu,
        init_tlb = sym crate::arch::init_tlb,
        entry = sym super::rust_entry,
        options(noreturn),
    )
}

/// The earliest entry point for secondary CPUs.
#[cfg(feature = "smp")]
#[naked]
#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn _start_secondary() -> ! {
    core::arch::asm!("
            ori          $t0, $zero, 0x1     # CSR_DMW1_PLV0
            lu52i.d      $t0, $t0, -2048     # UC, PLV0, 0x8000 xxxx xxxx xxxx
            csrwr        $t0, 0x180          # LOONGARCH_CSR_DMWIN0
            ori          $t0, $zero, 0x11    # CSR_DMW1_MAT | CSR_DMW1_PLV0
            lu52i.d      $t0, $t0, -1792     # CA, PLV0, 0x9000 xxxx xxxx xxxx
            csrwr        $t0, 0x181          # LOONGARCH_CSR_DMWIN1
            la.abs       $t0, {sm_boot_stack_top}
            ld.d         $sp, $t0,0          # read boot stack top

            csrrd $a0, 0x20                  # cpuid
            la.global $t0, {entry}
            jirl $zero,$t0,0
    ",
        sm_boot_stack_top = sym SMP_BOOT_STACK_TOP,
        entry = sym super::rust_entry_secondary,
        options(noreturn),
    )
}
