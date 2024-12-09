use memory_addr::VirtAddr;

use crate::arch::CacheOp;

/// Performs a cache operation on a range of memory.
pub fn dcache_range(_op: CacheOp, _addr: VirtAddr, _size: usize) {}

/// Performs a cache operation on a cache line.
#[inline]
pub fn dcache_line(_op: CacheOp, _addr: VirtAddr) {}
