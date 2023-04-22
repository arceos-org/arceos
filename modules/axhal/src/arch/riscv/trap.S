.macro SAVE_REGS, from_user             // 保存用户态的寄存器
    addi    sp, sp, -{trapframe_size}   
    PUSH_GENERAL_REGS                   // 内联汇编自带的指令             

    csrr    t0, sepc
    csrr    t1, sstatus
    csrrw   t2, sscratch, zero          // save sscratch (sp) and zero it
    // 代表此时再次发生trap就将会是S态的trap
    STR     t0, sp, 31                  // tf.sepc
    STR     t1, sp, 32                  // tf.sstatus
    STR     t2, sp, 1                   // tf.regs.sp

.if \from_user == 1
    LDR     t0, sp, 3                   // load supervisor tp
    STR     gp, sp, 2                   // save user gp and tp
    STR     tp, sp, 3                   
    mv      tp, t0                      // mv rd rs
.endif
.endm

.macro RESTORE_REGS, from_user
.if \from_user == 1
    LDR     gp, sp, 2                   // load user gp and tp
    LDR     t0, sp, 3
    STR     tp, sp, 3                   // save supervisor tp
    mv      tp, t0                      // tp：线程指针
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
    csrrw   sp, sscratch, sp            // switch sscratch and sp
    // 此时sscratch指向了用户栈高地址，sp指向了应用地址空间中trap上下文所在的位置
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
