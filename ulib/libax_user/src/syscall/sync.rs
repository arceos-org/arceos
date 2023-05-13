use crate::{syscall, syscall::sys_number::SYS_FUTEX};
pub use syscall_number::futex::*;

/*
long syscall(SYS_futex, uint32_t *uaddr, int futex_op, uint32_t val,
                    const struct timespec *timeout,   /* or: uint32_t val2 */
uint32_t *uaddr2, uint32_t val3);
*/
pub fn futex(uaddr: *const u32, futex_op: i32, val: u32,
             val2: u32, uaddr2: *const u32, val3: u32) {
    syscall(SYS_FUTEX, [uaddr as usize, futex_op as usize, val as usize, val2 as usize, uaddr2 as usize, val3 as usize]);
}
