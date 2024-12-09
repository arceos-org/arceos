use core::arch::asm;

use aarch64_cpu::{asm::barrier::*, registers::*};
use memory_addr::VirtAddr;

use crate::arch::CacheOp;

#[inline(always)]
fn cache_line_size() -> usize {
    unsafe {
        let mut ctr_el0: u64;
        asm!("mrs {}, ctr_el0", out(reg) ctr_el0);
        let log2_cache_line_size = ((ctr_el0 >> 16) & 0xF) as usize;
        // Calculate the cache line size
        4 << log2_cache_line_size
    }
}

/// Performs a cache operation on all memory.
pub fn dcache_all(op: CacheOp) {
    let clidr = CLIDR_EL1.get();

    for level in 0..8 {
        let ty = (clidr >> (level * 3)) & 0b111;
        if ty == 0 {
            return;
        }

        dcache_level(op, level);
    }
    dsb(SY);
    isb(SY);
}

/// Performs a cache operation on a range of memory.
#[inline]
pub fn dcache_range(op: CacheOp, addr: VirtAddr, size: usize) {
    let start = addr.as_usize();
    let end = start + size;
    let cache_line_size = cache_line_size();

    let mut aligned_addr = addr.as_usize() & !(cache_line_size - 1);

    while aligned_addr < end {
        _dcache_line(op, aligned_addr.into());
        aligned_addr += cache_line_size;
    }

    dsb(SY);
    isb(SY);
}

/// Performs a cache operation on a cache line.
#[inline]
fn _dcache_line(op: CacheOp, addr: VirtAddr) {
    unsafe {
        match op {
            CacheOp::Invalidate => asm!("dc ivac, {0:x}", in(reg) addr.as_usize()),
            CacheOp::Clean => asm!("dc cvac, {0:x}", in(reg) addr.as_usize()),
            CacheOp::CleanAndInvalidate => asm!("dc civac, {0:x}", in(reg) addr.as_usize()),
        }
    }
}

/// Performs a cache operation on a cache line.
#[inline]
pub fn dcache_line(op: CacheOp, addr: VirtAddr) {
    _dcache_line(op, addr);
    dsb(SY);
    isb(SY);
}

/// Performs a cache operation on a cache level.
/// https://developer.arm.com/documentation/ddi0601/2024-12/AArch64-Instructions/DC-CISW--Data-or-unified-Cache-line-Clean-and-Invalidate-by-Set-Way
#[inline]
fn dcache_level(op: CacheOp, level: u64) {
    assert!(level < 8, "armv8 level range is 0-7");

    isb(SY);
    CSSELR_EL1.write(CSSELR_EL1::InD::Data + CSSELR_EL1::Level.val(level));
    isb(SY);
    let lines = CCSIDR_EL1.read(CCSIDR_EL1::LineSize) as u32;
    let ways = CCSIDR_EL1.read(CCSIDR_EL1::AssociativityWithCCIDX) as u32;
    let sets = CCSIDR_EL1.read(CCSIDR_EL1::NumSetsWithCCIDX) as u32;

    let l = lines + 4;

    // Calculate bit position of number of ways
    let way_adjust = ways.leading_zeros();

    // Loop over sets and ways
    for set in 0..sets {
        for way in 0..ways {
            let set_way = (way << way_adjust) | set << l;

            let cisw = (level << 1) | set_way as u64;
            unsafe {
                match op {
                    CacheOp::Invalidate => asm!("dc isw, {0}", in(reg) cisw),
                    CacheOp::Clean => asm!("dc csw, {0}", in(reg) cisw),
                    CacheOp::CleanAndInvalidate => asm!("dc cisw, {0}", in(reg) cisw),
                }
            }
        }
    }
}
