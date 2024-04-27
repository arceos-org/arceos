use axconfig::TASK_STACK_SIZE;

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; TASK_STACK_SIZE] = [0; TASK_STACK_SIZE];
#[cfg(feature = "smp")]
pub static mut SMP_BOOT_STACK_TOP: usize = 0;

/// The earliest entry point for the primary CPU.
///
/// We can't use bl to jump to higher address, so we use jirl to jump to higher address.
#[naked]
#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn _start() -> ! {
    core::arch::asm!("
        csrrd       $t0, 0x20               # cpuid
        andi        $t0, $t0, 0x3ff         # cpuid & 0x3ff
        li.d        $a0, 0                  # boot core
        bne         $t0, $a0, 1f            # if cpuid != 0, goto slave_main
        
        ori         $t0, $zero, 0x1     # CSR_DMW1_PLV0
        lu52i.d     $t0, $t0, -2048     # UC, PLV0, 0x8000 xxxx xxxx xxxx
        csrwr       $t0, 0x180          # LOONGARCH_CSR_DMWIN0
        ori         $t0, $zero, 0x11    # CSR_DMW1_MAT | CSR_DMW1_PLV0
        lu52i.d     $t0, $t0, -1792     # CA, PLV0, 0x9000 xxxx xxxx xxxx
        csrwr       $t0, 0x181          # LOONGARCH_CSR_DMWIN1

        # Enable PG 
        li.w		$t0, 0xb0		# PLV=0, IE=0, PG=1
        csrwr		$t0, 0x0        # LOONGARCH_CSR_CRMD
        li.w		$t0, 0x00		# PLV=0, PIE=0, PWE=0
        csrwr		$t0, 0x1        # LOONGARCH_CSR_PRMD
        li.w		$t0, 0x00		# FPE=0, SXE=0, ASXE=0, BTE=0
        csrwr		$t0, 0x2        # LOONGARCH_CSR_EUEN

        la.global   $sp, {boot_stack}
        li.d        $t0, {boot_stack_size}
        add.d       $sp, $sp, $t0       # setup boot stack
        csrrd       $a0, 0x20           # cpuid
        la.global $t0, {entry}
        jirl $zero,$t0,0
        

        1:
        li.d        $t1, {mail_buf0}
        iocsrwr.d   $zero, $t1          # clear mail box 0
        
        li.d        $t0, (1<<12)
        csrxchg     $t0, $t0, 0x4       # set ECFG to enable IPI interrupt
        
        addi.d      $t0, $zero, -1
        li.d        $t1, {ipi_en}
        iocsrwr.w   $t0, $t1            # enable IPI interrupt
    
        li.d        $t1, {mail_buf0}
        
        2:
        idle 0
        nop
        iocsrrd.w   $t0, $t1            # read mail box 0
        beqz        $t0, 2b             # if mail box 0 is empty, goto 1b

        li.d        $t1, {ipi_status}   # read and clear ipi interrupt
        iocsrrd.w   $t0, $t1            
        li.d        $t1, {ipi_clear}
        iocsrwr.w   $t0, $t1
        
        li.d        $t1, (1<<12)
        csrxchg     $t0, $t0, 0x4       # disable ipi interrupt
        
        li.d        $t1, {mail_buf0}
        iocsrrd.d   $t0, $t1            # read mail box 0
        or          $ra, $t0, $zero     # set boot core flag
        jirl        $zero, $ra, 0       # jump to rust_entry
        ",
        boot_stack_size = const TASK_STACK_SIZE,
        boot_stack = sym BOOT_STACK,
        entry = sym super::rust_entry,
        mail_buf0 = const loongarch64::consts::LOONGARCH_CSR_MAIL_BUF0,
        ipi_en = const loongarch64::consts::LOONGARCH_IOCSR_IPI_EN,
        ipi_status = const loongarch64::consts::LOONGARCH_IOCSR_IPI_STATUS,
        ipi_clear = const loongarch64::consts::LOONGARCH_IOCSR_IPI_CLEAR,
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
