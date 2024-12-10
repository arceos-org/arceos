#![allow(unused)]

use core::{arch::asm, ptr::NonNull};

#[naked]
unsafe extern "C" fn _dcache_invalidate_range(_addr: usize, _end: usize) {
    asm!(
        "mrs	x3, ctr_el0",
        "ubfx	x3, x3, #16, #4",
        "mov	x2, #4",
        "lsl	x2, x2, x3", /* cache line size */
        /* x2 <- minimal cache line size in cache system */
        "sub	x3, x2, #1",
        "bic	x0, x0, x3",
        "1:	dc	ivac, x0", /* invalidate data or unified cache */
        "add	x0, x0, x2",
        "cmp	x0, x1",
        "b.lo	1b",
        "dsb	sy",
        "ret",
        options(noreturn)
    );
}

/// Invalidate data cache
pub fn dcache_invalidate_range(addr: NonNull<u8>, size: usize) {
    unsafe { _dcache_invalidate_range(addr.as_ptr() as usize, addr.as_ptr() as usize + size) }
}

#[naked]
unsafe extern "C" fn _dcache_flush_range(_addr: usize, _end: usize) {
    asm!(
        "mrs	x3, ctr_el0",
        "ubfx	x3, x3, #16, #4",
        "mov	x2, #4",
        "lsl	x2, x2, x3", /* cache line size */
        /* x2 <- minimal cache line size in cache system */
        "sub	x3, x2, #1",
        "bic	x0, x0, x3",
        "1:	dc	civac, x0", /* clean & invalidate data or unified cache */
        "add	x0, x0, x2",
        "cmp	x0, x1",
        "b.lo	1b",
        "dsb	sy",
        "ret",
        options(noreturn)
    );
}

/// Flush data cache
pub fn dcache_flush_range(addr: NonNull<u8>, size: usize) {
    unsafe { _dcache_flush_range(addr.as_ptr() as usize, addr.as_ptr() as usize + size) }
}
