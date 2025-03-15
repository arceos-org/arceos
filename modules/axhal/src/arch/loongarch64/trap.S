.macro SAVE_REGS, from_user
    move    $t0,  $sp
.if \from_user == 1
    csrrd   $sp,  KSAVE_KSP                   // restore kernel sp
    addi.d  $sp,  $sp, -{trapframe_size}

    STD     $tp,  $sp, 2
    STD     $r21, $sp, 21
    csrrd   $tp,  KSAVE_TP
    csrrd   $r21, KSAVE_R21
.else
    addi.d  $sp,  $sp, -{trapframe_size}
.endif

    STD     $t0,  $sp, 3
    csrrd   $t0,  KSAVE_TEMP
    PUSH_GENERAL_REGS
    csrrd   $t1,  LA_CSR_PRMD
    csrrd   $t2,  LA_CSR_ERA
    STD     $t1,  $sp, 32        // prmd
    STD     $t2,  $sp, 33        // era
.endm

.macro RESTORE_REGS, from_user
.if \from_user == 1
    csrwr   $tp,  KSAVE_TP
    csrwr   $r21, KSAVE_R21
    LDD     $tp,  $sp, 2
    LDD     $r21, $sp, 21
    addi.d  $t1,  $sp, {trapframe_size}
    csrwr   $t1,  KSAVE_KSP // save kernel sp
.endif

    LDD     $t1,  $sp, 33       // era
    LDD     $t2,  $sp, 32       // prmd
    csrwr   $t1,  LA_CSR_ERA
    csrwr   $t2,  LA_CSR_PRMD
    POP_GENERAL_REGS
    LDD     $sp,  $sp, 3
.endm

.section .text
.balign 4096
.global exception_entry_base
exception_entry_base:
    csrwr   $t0, KSAVE_TEMP
    csrrd   $t0, LA_CSR_PRMD
    andi    $t0, $t0, 0x3
    bnez    $t0, .Lfrom_userspace

.Lfrom_kernel:
    SAVE_REGS 0
    move    $a0, $sp
    addi.d  $a1, $zero, 0
    bl      loongarch64_trap_handler
    RESTORE_REGS 0
    ertn

.Lfrom_userspace:
    SAVE_REGS 1
    move    $a0, $sp
    addi.d  $a1, $zero, 1
    bl      loongarch64_trap_handler
    RESTORE_REGS 1
    ertn

.section .text
.balign 4096
.global handle_tlb_refill
handle_tlb_refill:
    csrwr   $t0, LA_CSR_TLBRSAVE
    csrrd   $t0, LA_CSR_PGD
    lddir   $t0, $t0, 3
    lddir   $t0, $t0, 2
    lddir   $t0, $t0, 1
    ldpte   $t0, 0
    ldpte   $t0, 1
    tlbfill
    csrrd   $t0, LA_CSR_TLBRSAVE
    ertn
