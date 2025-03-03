
.macro RESTORE_REGS
    csrrd   $t0, LA_CSR_PRMD
    andi    $t0, $t0, 0x3
    bnez    $t0, .Ltmp_user

.Ltmp_kernel:
    b .Ltmp_common

.Ltmp_user:
    csrwr   $tp,  KSAVE_TP
    csrwr   $r21, KSAVE_R21

    ld.d    $tp,  $sp, 2*8
    ld.d    $r21, $sp, 21*8

.Ltmp_common:
    POP_GENERAL_REGS

.endm

.section .text
.balign 4096
.global exception_entry_base
exception_entry_base:
    csrwr   $r21, KSAVE_TEMP
    csrrd   $r21, LA_CSR_PRMD
    andi    $r21, $r21, 0x3
    bnez    $r21, .Lfrom_userspace 

.Lfrom_kernel:
    move    $r21, $sp
    addi.d  $sp, $sp, -{trapframe_size} // allocate space
    // save kernel sp
    st.d    $r21, $sp, 3*8
    csrrd   $r21, KSAVE_TEMP
    b .Lcommon 

.Lfrom_userspace:
    csrwr   $sp, KSAVE_USP                   // save user sp into SAVE1 CSR
    csrrd   $sp, KSAVE_KSP                   // restore kernel sp
    addi.d  $sp, $sp, -{trapframe_size}      // allocate space

    // save user sp
    csrrd   $r21, KSAVE_USP
    st.d    $r21, $sp, 3*8 // sp

    // switch tp and r21
    st.d    $tp,  $sp, 2*8
    csrrd   $r21, KSAVE_TEMP
    st.d    $r21, $sp, 21*8

    csrrd   $tp,  KSAVE_TP
    csrrd   $r21, KSAVE_R21

.Lcommon:
    // save the registers.
    PUSH_GENERAL_REGS

    csrrd	$t2, LA_CSR_PRMD
    st.d	$t2, $sp, 8*32  // prmd
    csrrd   $t1, LA_CSR_ERA        
    st.d    $t1, $sp, 8*33  // era

    move    $a0, $sp
    andi    $a1, $t2, 0x3   // if user or kernel
    bl      loongarch64_trap_handler

    // restore the registers.
    ld.d    $t1, $sp, 8*33  // era
    csrwr   $t1, LA_CSR_ERA
    ld.d    $t2, $sp, 8*32  // prmd
    csrwr   $t2, LA_CSR_PRMD

    // Save kernel sp when exit kernel mode
    addi.d  $t1, $sp, {trapframe_size}
    csrwr   $t1, KSAVE_KSP 

    RESTORE_REGS

    // restore sp
    ld.d    $sp, $sp, 3*8
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
