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

    // restore sepc
    LDR     t0, sp, 31
    csrw    sepc, t0
    // restore sstatus, but don't change FS
    LDR     t0, sp, 32              // t0 = sstatus to restore
    csrr    t1, sstatus             // t1 = current sstatus
    li      t2, 0x6000              // t2 = mask for FS
    and     t1, t1, t2              // t1 = current FS
    not     t2, t2                  // t2 = ~(mask for FS)
    and     t0, t0, t2              // t0 = sstatus to restore(cleared FS)
    or      t0, t0, t1              // t0 = sstatus to restore with current FS
    csrw    sstatus, t0             // restore sstatus

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
