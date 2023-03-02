.macro SAVE_REGS
    sub     sp, sp, 34 * 8
    stp     x0, x1, [sp]
    stp     x2, x3, [sp, 2 * 8]
    stp     x4, x5, [sp, 4 * 8]
    stp     x6, x7, [sp, 6 * 8]
    stp     x8, x9, [sp, 8 * 8]
    stp     x10, x11, [sp, 10 * 8]
    stp     x12, x13, [sp, 12 * 8]
    stp     x14, x15, [sp, 14 * 8]
    stp     x16, x17, [sp, 16 * 8]
    stp     x18, x19, [sp, 18 * 8]
    stp     x20, x21, [sp, 20 * 8]
    stp     x22, x23, [sp, 22 * 8]
    stp     x24, x25, [sp, 24 * 8]
    stp     x26, x27, [sp, 26 * 8]
    stp     x28, x29, [sp, 28 * 8]

    mrs     x9, sp_el0
    mrs     x10, elr_el1
    mrs     x11, spsr_el1
    stp     x30, x9, [sp, 30 * 8]
    stp     x10, x11, [sp, 32 * 8]
.endm

.macro RESTORE_REGS
    ldp     x10, x11, [sp, 32 * 8]
    ldp     x30, x9, [sp, 30 * 8]
    msr     sp_el0, x9
    msr     elr_el1, x10
    msr     spsr_el1, x11

    ldp     x28, x29, [sp, 28 * 8]
    ldp     x26, x27, [sp, 26 * 8]
    ldp     x24, x25, [sp, 24 * 8]
    ldp     x22, x23, [sp, 22 * 8]
    ldp     x20, x21, [sp, 20 * 8]
    ldp     x18, x19, [sp, 18 * 8]
    ldp     x16, x17, [sp, 16 * 8]
    ldp     x14, x15, [sp, 14 * 8]
    ldp     x12, x13, [sp, 12 * 8]
    ldp     x10, x11, [sp, 10 * 8]
    ldp     x8, x9, [sp, 8 * 8]
    ldp     x6, x7, [sp, 6 * 8]
    ldp     x4, x5, [sp, 4 * 8]
    ldp     x2, x3, [sp, 2 * 8]
    ldp     x0, x1, [sp]
    add     sp, sp, 34 * 8
.endm

.macro INVALID_EXCP, kind, source
.p2align 7
    SAVE_REGS
    mov     x0, sp
    mov     x1, \kind
    mov     x2, \source
    bl      invalid_exception
    b       .Lexception_return
.endm

.macro HANDLE_SYNC
.p2align 7
    SAVE_REGS
    mov     x0, sp
    bl      handle_sync_exception
    b       .Lexception_return
.endm

.macro HANDLE_IRQ
.p2align 7
    SAVE_REGS
    mov     x0, sp
    bl      handle_irq_exception
    b       .Lexception_return
.endm

.section .text
.p2align 11
.global exception_vector_base
exception_vector_base:
    // current EL, with SP_EL0
    INVALID_EXCP 0 0
    INVALID_EXCP 1 0
    INVALID_EXCP 2 0
    INVALID_EXCP 3 0

    // current EL, with SP_ELx
    HANDLE_SYNC
    HANDLE_IRQ
    INVALID_EXCP 2 1
    INVALID_EXCP 3 1

    // lower EL, aarch64
    HANDLE_SYNC
    HANDLE_IRQ
    INVALID_EXCP 2 2
    INVALID_EXCP 3 2

    // lower EL, aarch32
    INVALID_EXCP 0 3
    INVALID_EXCP 1 3
    INVALID_EXCP 2 3
    INVALID_EXCP 3 3

.Lexception_return:
    RESTORE_REGS
    eret
