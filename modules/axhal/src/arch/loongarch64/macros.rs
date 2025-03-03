macro_rules! include_asm_macros {
    () => {
        r"
        .ifndef REGS_MACROS_FLAG
        .equ REGS_MACROS_FLAG, 1

        // CSR list
        .equ LA_CSR_PRMD, 0x1
        .equ LA_CSR_EUEN, 0x2
        .equ LA_CSR_ERA,  0x6
        .equ LA_CSR_PWCL, 0x1c
        .equ LA_CSR_PWCH, 0x1d

        .equ KSAVE_KSP, 0x30
        .equ KSAVE_TEMP,0x31
        .equ KSAVE_USP, 0x32
        .equ KSAVE_R21, 0x33
        .equ KSAVE_TP,  0x34

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

        .macro PUSH_POP_GENERAL_REGS, op
            \op    $ra, $sp, 1*8
            \op    $a0, $sp, 4*8
            \op    $a1, $sp, 5*8
            \op    $a2, $sp, 6*8
            \op    $a3, $sp, 7*8
            \op    $a4, $sp, 8*8
            \op    $a5, $sp, 9*8
            \op    $a6, $sp, 10*8
            \op    $a7, $sp, 11*8
            \op    $t0, $sp, 12*8
            \op    $t1, $sp, 13*8
            \op    $t2, $sp, 14*8
            \op    $t3, $sp, 15*8
            \op    $t4, $sp, 16*8
            \op    $t5, $sp, 17*8
            \op    $t6, $sp, 18*8
            \op    $t7, $sp, 19*8
            \op    $t8, $sp, 20*8
            \op    $fp, $sp, 22*8
            \op    $s0, $sp, 23*8
            \op    $s1, $sp, 24*8
            \op    $s2, $sp, 25*8
            \op    $s3, $sp, 26*8
            \op    $s4, $sp, 27*8
            \op    $s5, $sp, 28*8
            \op    $s6, $sp, 29*8
            \op    $s7, $sp, 30*8
            \op    $s8, $sp, 31*8
        .endm

        .macro PUSH_GENERAL_REGS
            PUSH_POP_GENERAL_REGS st.d
        .endm
        .macro POP_GENERAL_REGS
            PUSH_POP_GENERAL_REGS ld.d
        .endm

        .endif"
    };
}
