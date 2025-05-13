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

#[cfg(feature = "fp_simd")]
macro_rules! include_fp_asm_macros {
    () => {
        r#"
        .ifndef FP_MACROS_FLAG
        .equ FP_MACROS_FLAG, 1

        .macro SAVE_FCSR, base
            movfcsr2gr $t0, $fcsr0
            st.d $t0, \base, 0*8
        .endm
        .macro SAVE_FCC, base
            movcf2gr $t0, $fcc0
            movcf2gr $t1, $fcc1
            movcf2gr $t2, $fcc2
            movcf2gr $t3, $fcc3
            movcf2gr $t4, $fcc4
            movcf2gr $t5, $fcc5
            movcf2gr $t6, $fcc6
            movcf2gr $t7, $fcc7
            st.d $t0, \base, 0
            st.d $t1, \base, 1
            st.d $t2, \base, 2
            st.d $t3, \base, 3
            st.d $t4, \base, 4
            st.d $t5, \base, 5
            st.d $t6, \base, 6
            st.d $t7, \base, 7
        .endm

        .macro RESTORE_FCSR, base
            movgr2fcsr $fcsr0, $t0
            ld.d $t0, \base, 0*8
        .endm
        .macro RESTORE_FCC, base
            ld.d $t0, \base, 0
            ld.d $t1, \base, 1
            ld.d $t2, \base, 2
            ld.d $t3, \base, 3
            ld.d $t4, \base, 4
            ld.d $t5, \base, 5
            ld.d $t6, \base, 6
            ld.d $t7, \base, 7
            movgr2cf $fcc0, $t0
            movgr2cf $fcc1, $t1
            movgr2cf $fcc2, $t2
            movgr2cf $fcc3, $t3
            movgr2cf $fcc4, $t4
            movgr2cf $fcc5, $t5
            movgr2cf $fcc6, $t6
            movgr2cf $fcc7, $t7
        .endm


        // LoongArch64 specific floating point macros
        .macro PUSH_POP_FLOAT_REGS, op, base_reg
            \op $f0,  \base_reg, 0*8
            \op $f1,  \base_reg, 1*8
            \op $f2,  \base_reg, 2*8
            \op $f3,  \base_reg, 3*8
            \op $f4,  \base_reg, 4*8
            \op $f5,  \base_reg, 5*8
            \op $f6,  \base_reg, 6*8
            \op $f7,  \base_reg, 7*8
            \op $f8,  \base_reg, 8*8
            \op $f9,  \base_reg, 9*8
            \op $f10, \base_reg, 10*8
            \op $f11, \base_reg, 11*8
            \op $f12, \base_reg, 12*8
            \op $f13, \base_reg, 13*8
            \op $f14, \base_reg, 14*8
            \op $f15, \base_reg, 15*8
            \op $f16, \base_reg, 16*8
            \op $f17, \base_reg, 17*8
            \op $f18, \base_reg, 18*8
            \op $f19, \base_reg, 19*8
            \op $f20, \base_reg, 20*8
            \op $f21, \base_reg, 21*8
            \op $f22, \base_reg, 22*8
            \op $f23, \base_reg, 23*8
            \op $f24, \base_reg, 24*8
            \op $f25, \base_reg, 25*8
            \op $f26, \base_reg, 26*8
            \op $f27, \base_reg, 27*8
            \op $f28, \base_reg, 28*8
            \op $f29, \base_reg, 29*8
            \op $f30, \base_reg, 30*8
            \op $f31, \base_reg, 31*8
        .endm

        .macro PUSH_FLOAT_REGS, base_reg
            PUSH_POP_FLOAT_REGS fst.d, \base_reg
        .endm

        .macro POP_FLOAT_REGS, base_reg
            PUSH_POP_FLOAT_REGS fld.d, \base_reg
        .endm

        .endif"#
    };
}
