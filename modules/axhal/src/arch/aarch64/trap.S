.macro  clear_gp_regs
.irp    n,0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29
mov     x\n, xzr
.endr
.endm

.macro SAVE_REGS, el
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
    str     x30, [sp, 30 * 8]

    mrs     x10, elr_el1
    mrs     x11, spsr_el1

    stp     x10, x11, [sp, 32 * 8]

    .if  \el == 0
    clear_gp_regs
    mrs     x12, tpidr_el0  // save user tls pointer
     
    ldr     x13, [sp, 31 * 8] // restore current ktask ptr

    mrs     x9, sp_el0   // save user stack pointer */
    msr     sp_el0, x13   // restore kernel task ptr
    .else
    mov     x9, sp
    mov     x12, xzr
    .endif

    str     x12, [sp, 34 * 8] // save tpidr_el0
    str     x9, [sp, 31 * 8]  //  save user sp
.endm

.macro RESTORE_REGS, el
    ldp     x30, x9, [sp, 30 * 8]    // load user sp_el0
   
    ldp     x10, x11, [sp, 32 * 8]   // load ELR, SPSR
    msr     elr_el1, x10
    msr     spsr_el1, x11

    ldr     x12, [sp, 34 * 8]

    .if \el == 0
    msr     tpidr_el0, x12  // restore user tls pointer
   
    mrs     x13,  sp_el0    // save current ktask ptr
    str     x13,  [sp, 31 * 8]
    msr     sp_el0, x9     // restore user sp
    .endif

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
    add     sp, sp, 35 * 8
.endm

.macro HANDLE_TRAP, el, ht, regsize, label 
.p2align 7
    b  handle_el\el\ht\()_\regsize\()_\label
.endm

.macro HANDLE, el, ht, regsize, label
.section .text
handle_el\el\ht\()_\regsize\()_\label:
    sub     sp, sp, 35 * 8
    SAVE_REGS \el

    mov     x0, sp
    bl      handle_el\el\ht\()_\regsize\()_\label\()_exception 

    .if     \el == 1
	b ret_to_kernel
    .else
	b ret_to_user
    .endif
.endm

.section .text
.p2align 11
.global exception_vector_base
exception_vector_base:
    // current EL, with SP_EL0
    HANDLE_TRAP 1, t, 64, sync
    HANDLE_TRAP 1, t, 64, irq
    HANDLE_TRAP 1, t, 64, fiq
    HANDLE_TRAP 1, t, 64, error

    // current EL, with SP_ELx
    HANDLE_TRAP 1, h, 64, sync
    HANDLE_TRAP 1, h, 64, irq
    HANDLE_TRAP 1, h, 64, fiq
    HANDLE_TRAP 1, h, 64, error

    // lower EL, aarch64 with SP_EL0
    HANDLE_TRAP 0, t, 64, sync
    HANDLE_TRAP 0, t, 64, irq
    HANDLE_TRAP 0, t, 64, fiq
    HANDLE_TRAP 0, t, 64, error

    // lower EL, aarch32
    HANDLE_TRAP 0, t, 32, sync
    HANDLE_TRAP 0, t, 32, irq
    HANDLE_TRAP 0, t, 32, fiq
    HANDLE_TRAP 0, t, 32, error

/*
 * used to create handle_el_label_trap
*/
    // current EL, with SP_EL0
    HANDLE 1, t, 64, sync
    HANDLE 1, t, 64, irq
    HANDLE 1, t, 64, fiq
    HANDLE 1, t, 64, error

    // current EL, with SP_ELx
    HANDLE 1, h, 64, sync
    HANDLE 1, h, 64, irq
    HANDLE 1, h, 64, fiq
    HANDLE 1, h, 64, error

    // lower EL, aarch64 with SP_EL0
    HANDLE 0, t, 64, sync
    HANDLE 0, t, 64, irq
    HANDLE 0, t, 64, fiq
    HANDLE 0, t, 64, error

    // lower EL, aarch32
    HANDLE 0, t, 32, sync
    HANDLE 0, t, 32, irq
    HANDLE 0, t, 32, fiq
    HANDLE 0, t, 32, error

.section .text
.global ret_to_kernel
ret_to_kernel:
    RESTORE_REGS 1
    eret

.section .text
.global ret_to_user
ret_to_user:
    RESTORE_REGS 0
    eret

.section .text
.global ret_to_first_user
ret_to_first_user:
    mov sp, x0
    b ret_to_user
