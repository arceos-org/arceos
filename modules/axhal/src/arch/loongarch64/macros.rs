macro_rules! include_asm_macros {
    () => {
        r"
        .ifndef REGS_MACROS_FLAG
        .equ REGS_MACROS_FLAG, 1

        // CSR list
        .equ LA_CSR_PRMD,          0x1
        .equ LA_CSR_EUEN,          0x2
        .equ LA_CSR_ERA,           0x6
        .equ LA_CSR_PGDL,          0x19    // Page table base address when VA[47] = 0
        .equ LA_CSR_PGDH,          0x1a    // Page table base address when VA[47] = 1
        .equ LA_CSR_PGD,           0x1b    // Page table base
        .equ LA_CSR_PWCL,          0x1c
        .equ LA_CSR_PWCH,          0x1d
        .equ LA_CSR_TLBRENTRY,     0x88    // TLB refill exception entry
        .equ LA_CSR_TLBRBADV,      0x89    // TLB refill badvaddr
        .equ LA_CSR_TLBRERA,       0x8a    // TLB refill ERA
        .equ LA_CSR_TLBRSAVE,      0x8b    // KScratch for TLB refill exception
        .equ LA_CSR_TLBRELO0,      0x8c    // TLB refill entrylo0
        .equ LA_CSR_TLBRELO1,      0x8d    // TLB refill entrylo1
        .equ LA_CSR_TLBREHI,       0x8e    // TLB refill entryhi
        .equ LA_CSR_DMW0,          0x180
        .equ LA_CSR_DMW1,          0x181

        .equ KSAVE_KSP,            0x30
        .equ KSAVE_TEMP,           0x31
        .equ KSAVE_R21,            0x32
        .equ KSAVE_TP,             0x33

        .macro STD rd, rj, off
            st.d   \rd, \rj, \off*8
        .endm
        .macro LDD rd, rj, off
            ld.d   \rd, \rj, \off*8
        .endm

        .macro PUSH_POP_GENERAL_REGS, op
            \op    $ra, $sp, 1
            \op    $a0, $sp, 4
            \op    $a1, $sp, 5
            \op    $a2, $sp, 6
            \op    $a3, $sp, 7
            \op    $a4, $sp, 8
            \op    $a5, $sp, 9
            \op    $a6, $sp, 10
            \op    $a7, $sp, 11
            \op    $t0, $sp, 12
            \op    $t1, $sp, 13
            \op    $t2, $sp, 14
            \op    $t3, $sp, 15
            \op    $t4, $sp, 16
            \op    $t5, $sp, 17
            \op    $t6, $sp, 18
            \op    $t7, $sp, 19
            \op    $t8, $sp, 20
            \op    $fp, $sp, 22
            \op    $s0, $sp, 23
            \op    $s1, $sp, 24
            \op    $s2, $sp, 25
            \op    $s3, $sp, 26
            \op    $s4, $sp, 27
            \op    $s5, $sp, 28
            \op    $s6, $sp, 29
            \op    $s7, $sp, 30
            \op    $s8, $sp, 31
        .endm

        .macro PUSH_GENERAL_REGS
            PUSH_POP_GENERAL_REGS STD
        .endm
        .macro POP_GENERAL_REGS
            PUSH_POP_GENERAL_REGS LDD
        .endm

        .endif"
    };
}
