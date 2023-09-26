    .section .text
    .global __alter_trap_read_at
    .global __alter_trap_write_at
    .global __alter_trap_entry

// a0: [input] read addr; [output] value to be read
// a1: [output] 0 if ok, scause if trapped
__alter_trap_read_at:
    #mv a1, zero
    ld a0, 0(a0)
    ret

// a0: [input] write addr
// a1: [output] 0 if ok, scause if trapped
// a2: [input] value to be write
__alter_trap_write_at:
    #mv a1, zero
    sd a2, 0(a0)
    ret

// if trapped, write scause to a1,
// and return next intr of __alter_trap_(read/write)_at
__alter_trap_entry:
    csrw sepc, ra   # ra -> __try_x_user_u8's return addr
    csrr a1, scause
    sret