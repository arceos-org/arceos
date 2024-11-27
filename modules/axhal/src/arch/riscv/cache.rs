#![allow(unused)]

use core::ptr::NonNull;

/// Invalidate data cache
pub fn dcache_invalidate_range(_addr: NonNull<u8>, _size: usize) {
    unimplemented!();
}

/// Flush data cache
pub fn dcache_flush_range(_addr: NonNull<u8>, _size: usize) {
    unimplemented!();
}
