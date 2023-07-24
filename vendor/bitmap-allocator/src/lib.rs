//! Bit allocator based on segment tree algorithm.

#![no_std]

use bit_field::BitField;
use core::ops::Range;

/// Allocator of a bitmap, able to allocate / free bits.
pub trait BitAlloc: Default {
    /// The bitmap has a total of CAP bits, numbered from 0 to CAP-1 inclusively.
    const CAP: usize;

    /// The default value. Workaround for `const fn new() -> Self`.
    const DEFAULT: Self;

    /// Allocate a free bit.
    fn alloc(&mut self) -> Option<usize>;

    /// Allocate a free block with a given size, and return the first bit position.
    fn alloc_contiguous(&mut self, size: usize, align_log2: usize) -> Option<usize>;

    /// Find a index not less than a given key, where the bit is free.
    fn next(&self, key: usize) -> Option<usize>;

    /// Free an allocated bit.
    fn dealloc(&mut self, key: usize);

    /// Mark bits in the range as unallocated (available)
    fn insert(&mut self, range: Range<usize>);

    /// Reverse of insert
    fn remove(&mut self, range: Range<usize>);

    /// Whether there are free bits remaining
    #[deprecated = "use `!self.is_empty()` instead"]
    fn any(&self) -> bool;

    /// Returns true if no bits is available.
    fn is_empty(&self) -> bool;

    /// Whether a specific bit is free
    fn test(&self, key: usize) -> bool;
}

/// A bitmap of 256 bits
pub type BitAlloc256 = BitAllocCascade16<BitAlloc16>;
/// A bitmap of 4K bits
pub type BitAlloc4K = BitAllocCascade16<BitAlloc256>;
/// A bitmap of 64K bits
pub type BitAlloc64K = BitAllocCascade16<BitAlloc4K>;
/// A bitmap of 1M bits
pub type BitAlloc1M = BitAllocCascade16<BitAlloc64K>;
/// A bitmap of 16M bits
pub type BitAlloc16M = BitAllocCascade16<BitAlloc1M>;
/// A bitmap of 256M bits
pub type BitAlloc256M = BitAllocCascade16<BitAlloc16M>;

/// Implement the bit allocator by segment tree algorithm.
#[derive(Default)]
pub struct BitAllocCascade16<T: BitAlloc> {
    bitset: u16, // for each bit, 1 indicates available, 0 indicates inavailable
    sub: [T; 16],
}

impl<T: BitAlloc> BitAlloc for BitAllocCascade16<T> {
    const CAP: usize = T::CAP * 16;

    const DEFAULT: Self = BitAllocCascade16 {
        bitset: 0,
        sub: [T::DEFAULT; 16],
    };

    fn alloc(&mut self) -> Option<usize> {
        if !self.is_empty() {
            let i = self.bitset.trailing_zeros() as usize;
            let res = self.sub[i].alloc().unwrap() + i * T::CAP;
            self.bitset.set_bit(i, !self.sub[i].is_empty());
            Some(res)
        } else {
            None
        }
    }
    fn alloc_contiguous(&mut self, size: usize, align_log2: usize) -> Option<usize> {
        if let Some(base) = find_contiguous(self, Self::CAP, size, align_log2) {
            self.remove(base..base + size);
            Some(base)
        } else {
            None
        }
    }
    fn dealloc(&mut self, key: usize) {
        let i = key / T::CAP;
        self.sub[i].dealloc(key % T::CAP);
        self.bitset.set_bit(i, true);
    }
    fn insert(&mut self, range: Range<usize>) {
        self.for_range(range, |sub: &mut T, range| sub.insert(range));
    }
    fn remove(&mut self, range: Range<usize>) {
        self.for_range(range, |sub: &mut T, range| sub.remove(range));
    }
    fn any(&self) -> bool {
        !self.is_empty()
    }
    fn is_empty(&self) -> bool {
        self.bitset == 0
    }
    fn test(&self, key: usize) -> bool {
        self.sub[key / T::CAP].test(key % T::CAP)
    }
    fn next(&self, key: usize) -> Option<usize> {
        let idx = key / T::CAP;
        (idx..16).find_map(|i| {
            if self.bitset.get_bit(i) {
                let key = if i == idx { key - T::CAP * idx } else { 0 };
                self.sub[i].next(key).map(|x| x + T::CAP * i)
            } else {
                None
            }
        })
    }
}

impl<T: BitAlloc> BitAllocCascade16<T> {
    fn for_range(&mut self, range: Range<usize>, f: impl Fn(&mut T, Range<usize>)) {
        let Range { start, end } = range;
        assert!(start <= end);
        assert!(end <= Self::CAP);
        for i in start / T::CAP..=(end - 1) / T::CAP {
            let begin = if start / T::CAP == i {
                start % T::CAP
            } else {
                0
            };
            let end = if end / T::CAP == i {
                end % T::CAP
            } else {
                T::CAP
            };
            f(&mut self.sub[i], begin..end);
            self.bitset.set_bit(i, !self.sub[i].is_empty());
        }
    }
}

/// A bitmap consisting of only 16 bits.
/// BitAlloc16 acts as the leaf (except the leaf bits of course) nodes in the segment trees.
#[derive(Default)]
pub struct BitAlloc16(u16);

impl BitAlloc for BitAlloc16 {
    const CAP: usize = u16::BITS as usize;

    const DEFAULT: Self = Self(0);

    fn alloc(&mut self) -> Option<usize> {
        let i = self.0.trailing_zeros() as usize;
        if i < Self::CAP {
            self.0.set_bit(i, false);
            Some(i)
        } else {
            None
        }
    }
    fn alloc_contiguous(&mut self, size: usize, align_log2: usize) -> Option<usize> {
        if let Some(base) = find_contiguous(self, Self::CAP, size, align_log2) {
            self.remove(base..base + size);
            Some(base)
        } else {
            None
        }
    }
    fn dealloc(&mut self, key: usize) {
        assert!(!self.test(key));
        self.0.set_bit(key, true);
    }
    fn insert(&mut self, range: Range<usize>) {
        self.0.set_bits(range.clone(), 0xffff.get_bits(range));
    }
    fn remove(&mut self, range: Range<usize>) {
        self.0.set_bits(range, 0);
    }
    fn any(&self) -> bool {
        !self.is_empty()
    }
    fn is_empty(&self) -> bool {
        self.0 == 0
    }
    fn test(&self, key: usize) -> bool {
        self.0.get_bit(key)
    }
    fn next(&self, key: usize) -> Option<usize> {
        (key..Self::CAP).find(|&i| self.0.get_bit(i))
    }
}

fn find_contiguous(
    ba: &impl BitAlloc,
    capacity: usize,
    size: usize,
    align_log2: usize,
) -> Option<usize> {
    if capacity < (1 << align_log2) || ba.is_empty() {
        return None;
    }
    let mut base = 0;
    let mut offset = base;
    while offset < capacity {
        if let Some(next) = ba.next(offset) {
            if next != offset {
                // it can be guarenteed that no bit in (offset..next) is free
                // move to next aligned position after next-1
                assert!(next > offset);
                base = (((next - 1) >> align_log2) + 1) << align_log2;
                assert_ne!(offset, next);
                offset = base;
                continue;
            }
        } else {
            return None;
        }
        offset += 1;
        if offset - base == size {
            return Some(base);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitalloc16() {
        let mut ba = BitAlloc16::default();
        assert_eq!(BitAlloc16::CAP, 16);
        ba.insert(0..16);
        for i in 0..16 {
            assert!(ba.test(i));
        }
        ba.remove(2..8);
        assert_eq!(ba.alloc(), Some(0));
        assert_eq!(ba.alloc(), Some(1));
        assert_eq!(ba.alloc(), Some(8));
        ba.dealloc(0);
        ba.dealloc(1);
        ba.dealloc(8);

        assert!(!ba.is_empty());
        for _ in 0..10 {
            assert!(ba.alloc().is_some());
        }
        assert!(ba.is_empty());
        assert!(ba.alloc().is_none());
    }

    #[test]
    fn bitalloc4k() {
        let mut ba = BitAlloc4K::default();
        assert_eq!(BitAlloc4K::CAP, 4096);
        ba.insert(0..4096);
        for i in 0..4096 {
            assert!(ba.test(i));
        }
        ba.remove(2..4094);
        for i in 0..4096 {
            assert_eq!(ba.test(i), !(2..4094).contains(&i));
        }
        assert_eq!(ba.alloc(), Some(0));
        assert_eq!(ba.alloc(), Some(1));
        assert_eq!(ba.alloc(), Some(4094));
        ba.dealloc(0);
        ba.dealloc(1);
        ba.dealloc(4094);

        assert!(!ba.is_empty());
        for _ in 0..4 {
            assert!(ba.alloc().is_some());
        }
        assert!(ba.is_empty());
        assert!(ba.alloc().is_none());
    }

    #[test]
    fn bitalloc_contiguous() {
        let mut ba0 = BitAlloc16::default();
        ba0.insert(0..BitAlloc16::CAP);
        ba0.remove(3..6);
        assert_eq!(ba0.next(0), Some(0));
        assert_eq!(ba0.alloc_contiguous(1, 1), Some(0));
        assert_eq!(find_contiguous(&ba0, BitAlloc4K::CAP, 2, 0), Some(1));

        let mut ba = BitAlloc4K::default();
        assert_eq!(BitAlloc4K::CAP, 4096);
        ba.insert(0..BitAlloc4K::CAP);
        ba.remove(3..6);
        assert_eq!(ba.next(0), Some(0));
        assert_eq!(ba.alloc_contiguous(1, 1), Some(0));
        assert_eq!(ba.next(0), Some(1));
        assert_eq!(ba.next(1), Some(1));
        assert_eq!(ba.next(2), Some(2));
        assert_eq!(find_contiguous(&ba, BitAlloc4K::CAP, 2, 0), Some(1));
        assert_eq!(ba.alloc_contiguous(2, 0), Some(1));
        assert_eq!(ba.alloc_contiguous(2, 3), Some(8));
        ba.remove(0..4096 - 64);
        assert_eq!(ba.alloc_contiguous(128, 7), None);
        assert_eq!(ba.alloc_contiguous(7, 3), Some(4096 - 64));
        ba.insert(321..323);
        assert_eq!(ba.alloc_contiguous(2, 1), Some(4096 - 64 + 8));
        assert_eq!(ba.alloc_contiguous(2, 0), Some(321));
        assert_eq!(ba.alloc_contiguous(64, 6), None);
        assert_eq!(ba.alloc_contiguous(32, 4), Some(4096 - 48));
        for i in 0..4096 - 64 + 7 {
            ba.dealloc(i);
        }
        for i in 4096 - 64 + 8..4096 - 64 + 10 {
            ba.dealloc(i);
        }
        for i in 4096 - 48..4096 - 16 {
            ba.dealloc(i);
        }
        // for i in 4096 - 32..4096 {
        //     ba.dealloc(i);
        // }
    }

    // #[test]
    // fn bitallocContPerformance() {
    //     let mut ba = Box::new(BitAlloc256M::default());
    //     assert_eq!(BitAlloc256M::CAP, 1 << 28);
    //     ba.insert(0..BitAlloc256M::CAP);
    //     assert_eq!(ba.alloc_contiguous(1 << 20, 20), Some(0));
    //     assert_eq!(ba.alloc_contiguous(1 << 19, 19), Some(1 << 20));
    //     assert_eq!(ba.alloc_contiguous(1 << 21, 21), Some(1 << 21));
    //     assert_eq!(ba.alloc_contiguous(1 << 19, 19), Some(3 << 19));
    // }
}
