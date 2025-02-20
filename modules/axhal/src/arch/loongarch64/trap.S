.equ KSAVE_KSP, 0x30
.equ KSAVE_T0,  0x31
.equ KSAVE_USP, 0x32
.equ KSAVE_R21, 0x33
.equ KSAVE_TP,  0x34

.macro SAVE_REGS
    st.d    $ra, $sp, 8
    csrrd   $t0, KSAVE_T0
    st.d    $t0, $sp, 12*8

    st.d    $a0, $sp, 4*8
    st.d    $a1, $sp, 5*8
    st.d    $a2, $sp, 6*8
    st.d    $a3, $sp, 7*8
    st.d    $a4, $sp, 8*8
    st.d    $a5, $sp, 9*8
    st.d    $a6, $sp, 10*8
    st.d    $a7, $sp, 11*8
    st.d    $t1, $sp, 13*8
    st.d    $t2, $sp, 14*8
    st.d    $t3, $sp, 15*8
    st.d    $t4, $sp, 16*8
    st.d    $t5, $sp, 17*8
    st.d    $t6, $sp, 18*8
    st.d    $t7, $sp, 19*8
    st.d    $t8, $sp, 20*8

    st.d    $fp, $sp, 22*8
    st.d    $s0, $sp, 23*8
    st.d    $s1, $sp, 24*8
    st.d    $s2, $sp, 25*8
    st.d    $s3, $sp, 26*8
    st.d    $s4, $sp, 27*8
    st.d    $s5, $sp, 28*8
    st.d    $s6, $sp, 29*8
    st.d    $s7, $sp, 30*8
    st.d    $s8, $sp, 31*8
.endm

.macro RESTORE_REGS
    csrrd   $t0, 0x1
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
    ld.d    $ra, $sp, 1*8
    ld.d    $a0, $sp, 4*8
    ld.d    $a1, $sp, 5*8
    ld.d    $a2, $sp, 6*8
    ld.d    $a3, $sp, 7*8
    ld.d    $a4, $sp, 8*8
    ld.d    $a5, $sp, 9*8
    ld.d    $a6, $sp, 10*8
    ld.d    $a7, $sp, 11*8
    ld.d    $t0, $sp, 12*8
    ld.d    $t1, $sp, 13*8
    ld.d    $t2, $sp, 14*8
    ld.d    $t3, $sp, 15*8
    ld.d    $t4, $sp, 16*8
    ld.d    $t5, $sp, 17*8
    ld.d    $t6, $sp, 18*8
    ld.d    $t7, $sp, 19*8
    ld.d    $t8, $sp, 20*8

    ld.d    $fp, $sp, 22*8
    ld.d    $s0, $sp, 23*8
    ld.d    $s1, $sp, 24*8
    ld.d    $s2, $sp, 25*8
    ld.d    $s3, $sp, 26*8
    ld.d    $s4, $sp, 27*8
    ld.d    $s5, $sp, 28*8
    ld.d    $s6, $sp, 29*8
    ld.d    $s7, $sp, 30*8
    ld.d    $s8, $sp, 31*8
.endm


.section .text
.balign 4096
.global trap_vector_base
trap_vector_base:
    csrwr   $t0, KSAVE_T0
    csrrd   $t0, 0x1
    andi    $t0, $t0, 0x3
    bnez    $t0, .Lfrom_userspace 

.Lfrom_kernel:
    move    $t0, $sp  
    addi.d  $sp, $sp, -{trapframe_size} // allocate space
    // save kernel sp
    st.d    $t0, $sp, 3*8

    b .Lcommon 

.Lfrom_userspace:       
    csrwr   $sp, KSAVE_USP                   // save user sp into SAVE1 CSR
    csrrd   $sp, KSAVE_KSP                   // restore kernel sp
    addi.d  $sp, $sp, -{trapframe_size}      // allocate space

    // switch tp and r21
    st.d    $tp,  $sp, 2*8
    st.d    $r21, $sp, 21*8

    csrrd   $tp,  KSAVE_TP
    csrrd   $r21, KSAVE_R21

    // save user sp
    csrrd   $t0, KSAVE_USP
    st.d    $t0, $sp, 3*8 // sp

.Lcommon:
    // save the registers.
    SAVE_REGS

    csrrd	$t2, 0x1
    st.d	$t2, $sp, 8*32  // prmd
    csrrd   $t1, 0x6        
    st.d    $t1, $sp, 8*33  // era
    csrrd   $t1, 0x7   
    st.d    $t1, $sp, 8*34  // badv  
    csrrd   $t1, 0x0   
    st.d    $t1, $sp, 8*35  // crmd    

    move    $a0, $sp
    csrrd   $t0, 0x1
    andi    $a1, $t0, 0x3   // if user or kernel
    bl      loongarch64_trap_handler

    // restore the registers.
    ld.d    $t1, $sp, 8*33  // era
    csrwr   $t1, 0x6
    ld.d    $t2, $sp, 8*32  // prmd
    csrwr   $t2, 0x1

    // Save kernel sp when exit kernel mode
    addi.d  $t1, $sp, {trapframe_size}
    csrwr   $t1, KSAVE_KSP 

    RESTORE_REGS

    // restore sp
    ld.d    $sp, $sp, 3*8
    ertn


// TLB Refill handler
.equ LA_CSR_PGDL,          0x19    /* Page table base address when VA[47] = 0 */
.equ LA_CSR_PGDH,          0x1a    /* Page table base address when VA[47] = 1 */
.equ LA_CSR_PGD,           0x1b    /* Page table base */
.equ LA_CSR_TLBRENTRY,     0x88    /* TLB refill exception entry */
.equ LA_CSR_TLBRBADV,      0x89    /* TLB refill badvaddr */
.equ LA_CSR_TLBRERA,       0x8a    /* TLB refill ERA */
.equ LA_CSR_TLBRSAVE,      0x8b    /* KScratch for TLB refill exception */
.equ LA_CSR_TLBRELO0,      0x8c    /* TLB refill entrylo0 */
.equ LA_CSR_TLBRELO1,      0x8d    /* TLB refill entrylo1 */
.equ LA_CSR_TLBREHI,       0x8e    /* TLB refill entryhi */

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
