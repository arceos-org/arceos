.macro PUSH_POP_GENERAL_REGS, op
    \op ra, sp, 0
    \op t0, sp, 4
    \op t1, sp, 5
    \op t2, sp, 6
    \op s0, sp, 7
    \op s1, sp, 8
    \op a0, sp, 9
    \op a1, sp, 10
    \op a2, sp, 11
    \op a3, sp, 12
    \op a4, sp, 13
    \op a5, sp, 14
    \op a6, sp, 15
    \op a7, sp, 16
    \op s2, sp, 17
    \op s3, sp, 18
    \op s4, sp, 19
    \op s5, sp, 20
    \op s6, sp, 21
    \op s7, sp, 22
    \op s8, sp, 23
    \op s9, sp, 24
    \op s10, sp, 25
    \op s11, sp, 26
    \op t3, sp, 27
    \op t4, sp, 28
    \op t5, sp, 29
    \op t6, sp, 30
.endm

.macro PUSH_GENERAL_REGS
    PUSH_POP_GENERAL_REGS STR
.endm

.macro POP_GENERAL_REGS
    PUSH_POP_GENERAL_REGS LDR
.endm

.macro SAVE_REGS, from_user
    addi    sp, sp, -{trapframe_size}
    PUSH_GENERAL_REGS

    csrr    t0, sepc
    csrr    t1, sstatus
    csrrw   t2, sscratch, zero          // save sscratch (sp) and zero it
    STR     t0, sp, 31                  // tf.sepc
    STR     t1, sp, 32                  // tf.sstatus
    STR     t2, sp, 1                   // tf.regs.sp

.if \from_user == 1
    LDR     t0, sp, 2                   // load supervisor gp
    LDR     t1, sp, 3                   // load supervisor tp
    STR     gp, sp, 2                   // save user gp and tp
    STR     tp, sp, 3
    mv      gp, t0
    mv      tp, t1
.endif
.endm

.macro RESTORE_REGS, from_user
.if \from_user == 1
    LDR     t1, sp, 2                   // load user gp and tp
    LDR     t0, sp, 3
    STR     gp, sp, 2                   // save supervisor gp
    STR     tp, sp, 3                   // save supervisor gp and tp
    mv      gp, t1
    mv      tp, t0
    addi    t0, sp, {trapframe_size}    // put supervisor sp to scratch
    csrw    sscratch, t0
.endif

    LDR     t0, sp, 31
    LDR     t1, sp, 32
    csrw    sepc, t0
    csrw    sstatus, t1

    POP_GENERAL_REGS
    LDR     sp, sp, 1                   // load sp from tf.regs.sp
.endm

.section .text
.balign 4
.global trap_vector_base
trap_vector_base:
    // sscratch == 0: trap from S mode
    // sscratch != 0: trap from U mode
    csrrw   sp, sscratch, sp            // swap sscratch and sp
    bnez    sp, .Ltrap_entry_u

    csrr    sp, sscratch                // put supervisor sp back
    j       .Ltrap_entry_s

.Ltrap_entry_s:
    SAVE_REGS 0
    mv      a0, sp
    li      a1, 0
    call    riscv_trap_handler
    RESTORE_REGS 0
    sret

.Ltrap_entry_u:
    SAVE_REGS 1
    mv      a0, sp
    li      a1, 1
    call    riscv_trap_handler
    RESTORE_REGS 1
    sret
