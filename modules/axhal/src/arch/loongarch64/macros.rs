macro_rules! include_asm_macros {
    () => {
        concat!(
            r"
            .ifndef REGS_MACROS_FLAG
            .equ REGS_MACROS_FLAG, 1

            // CSR list
            .equ LA_CSR_PRMD, 0x1
            .equ LA_CSR_EUEN, 0x2

            .equ KSAVE_KSP, 0x30
            .equ KSAVE_T0,  0x31
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

            .endif"
        )
    };
}
