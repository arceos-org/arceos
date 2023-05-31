.altmacro
.macro COPY n
    ld t2, (\n)*8(a0)
    sd t2, (\n)*8(a1)
.endm
    .section .text
    .globl __copy
__copy:
    # __copy(
    #     frame_address: *const TrapFrame,
    #     kernel_base: *mut T
    # )
    .set n, 0
    .rept 33
        COPY %n
        .set n, n + 1
    .endr
    ret

