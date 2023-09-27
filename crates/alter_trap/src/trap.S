    .section .text
// a0: [input] read addr; [output] value to be read
// a1: [input] should be 0; [output] 0 if ok, scause if trapped
.type __alter_trap_read_usize, %function
__alter_trap_read_usize:
    #mv a1, zero
    ld a0, 0(a0)
    ret

// a0: [input] write addr
// a1: [input] should be 0; [output] 0 if ok, scause if trapped
// a2: [input] value to be write
.type __alter_trap_write_usize, %function
__alter_trap_write_usize:
    #mv a1, zero
    sd a2, 0(a0)
    ret

// a0: [input] read/write addr; [output] value to be read
// a1: [input] should be 0; [output] 0 if ok, scause if trapped
.type __alter_trap_read_write_usize, %function
__alter_trap_read_write_usize:
    #mv a1, zero
    mv a2, a0
    ld a0, 0(a2)
    sd a1, 0(a2)
    sd a0, 0(a2)
    ret


// a0: [input] read addr; [output] value to be read
// a1: [input] should be 0; [output] 0 if ok, scause if trapped
.type __alter_trap_read_u8, %function
__alter_trap_read_u8:
    #mv a1, zero
    lb a0, 0(a0)
    ret

// a0: [input] write addr
// a1: [input] should be 0; [output] 0 if ok, scause if trapped
// a2: [input] value to be write
.type __alter_trap_write_u8, %function
__alter_trap_write_u8:
    #mv a1, zero
    sb a2, 0(a0)
    ret

// a0: [input] start addr of slice;
// a1: [input] should be 0; [output] 0 if ok, scause if trapped
// a2: [input] end addr of slice;
.type __alter_trap_check_slice_readable, %function
__alter_trap_check_slice_readable:
    #mv a1, zero
    lb t0, 0(a0) # try read at start point of slice
    lui t1, 0x1000 # page size
    sub t0, zero, t1 # mask of page size, =0xFFFFFFFFFFFFF000
    and a0, a0, t0 
    add a0, a0, t1 # a0 switch to next page
.Lcheck_loop_read:
    bltu a2, a0, .Lcheck_end_read # check if a0 crossed endpoint a2
    lb t0, 0(a0)
    add a0, a0, t1 # a0 switch to next page
    j .Lcheck_loop_read
.Lcheck_end_read:
    ret

// a0: [input] start addr of slice;
// a1: [input] should be 0; [output] 0 if ok, scause if trapped
// a2: [input] end addr of slice;
.type __alter_trap_check_slice_writable, %function
__alter_trap_check_slice_writable:
    #mv a1, zero
    sb a1, 0(a0) # try write at start point of slice
    # we must carefully write BYTE instand of DWORD, 
    # cuz the other bytes may belong to other objects.    
    lui t1, 0x1000 # page size
    sub t0, zero, t1 # mask of page size, =0xFFFFFFFFFFFFF000
    and a0, a0, t0 
    add a0, a0, t1 # a0 switch to next page
.Lcheck_loop_write:
    bltu a2, a0, .Lcheck_end_write # check if a0 crossed endpoint a2
    sb a1, 0(a0)
    add a0, a0, t1 # a0 switch to next page
    j .Lcheck_loop_write
.Lcheck_end_write:
    ret

// if trapped, write scause to a1,
// and return next intr of __alter_trap_(read/write)_at

.type __alter_trap_entry, %function
.align 2
__alter_trap_entry:
    csrw sepc, ra   # ra -> __try_x_user_u8's return addr
    csrr a1, scause
    sret