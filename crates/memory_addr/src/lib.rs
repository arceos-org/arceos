#![no_std]
#![feature(const_mut_refs)]
#![doc = include_str!("../README.md")]

use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// The size of a 4K page (4096 bytes).
pub const PAGE_SIZE_4K: usize = 0x1000;

/// Align address downwards.
///
/// Returns the greatest `x` with alignment `align` so that `x <= addr`.
///
/// The alignment must be a power of two.
#[inline]
pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

/// Align address upwards.
///
/// Returns the smallest `x` with alignment `align` so that `x >= addr`.
///
/// The alignment must be a power of two.
#[inline]
pub const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Returns the offset of the address within the alignment.
///
/// Equivalent to `addr % align`, but the alignment must be a power of two.
#[inline]
pub const fn align_offset(addr: usize, align: usize) -> usize {
    addr & (align - 1)
}

/// Checks whether the address has the demanded alignment.
///
/// Equivalent to `addr % align == 0`, but the alignment must be a power of two.
#[inline]
pub const fn is_aligned(addr: usize, align: usize) -> bool {
    align_offset(addr, align) == 0
}

/// Align address downwards to 4096 (bytes).
#[inline]
pub const fn align_down_4k(addr: usize) -> usize {
    align_down(addr, PAGE_SIZE_4K)
}

/// Align address upwards to 4096 (bytes).
#[inline]
pub const fn align_up_4k(addr: usize) -> usize {
    align_up(addr, PAGE_SIZE_4K)
}

/// Returns the offset of the address within a 4K-sized page.
#[inline]
pub const fn align_offset_4k(addr: usize) -> usize {
    align_offset(addr, PAGE_SIZE_4K)
}

/// Checks whether the address is 4K-aligned.
#[inline]
pub const fn is_aligned_4k(addr: usize) -> bool {
    is_aligned(addr, PAGE_SIZE_4K)
}

/// A physical memory address.
///
/// It's a wrapper type around an `usize`.
#[repr(transparent)]
#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(usize);

/// A virtual memory address.
///
/// It's a wrapper type around an `usize`.
#[repr(transparent)]
#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(usize);

impl PhysAddr {
    /// Converts an `usize` to a physical address.
    #[inline]
    pub const fn from(addr: usize) -> Self {
        Self(addr)
    }

    /// Converts the address to an `usize`.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0
    }

    /// Aligns the address downwards to the given alignment.
    ///
    /// See the [`align_down`] function for more information.
    #[inline]
    pub fn align_down<U>(self, align: U) -> Self
    where
        U: Into<usize>,
    {
        Self(align_down(self.0, align.into()))
    }

    /// Aligns the address upwards to the given alignment.
    ///
    /// See the [`align_up`] function for more information.
    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
    where
        U: Into<usize>,
    {
        Self(align_up(self.0, align.into()))
    }

    /// Returns the offset of the address within the given alignment.
    ///
    /// See the [`align_offset`] function for more information.
    #[inline]
    pub fn align_offset<U>(self, align: U) -> usize
    where
        U: Into<usize>,
    {
        align_offset(self.0, align.into())
    }

    /// Checks whether the address has the demanded alignment.
    ///
    /// See the [`is_aligned`] function for more information.
    #[inline]
    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<usize>,
    {
        is_aligned(self.0, align.into())
    }

    /// Aligns the address downwards to 4096 (bytes).
    #[inline]
    pub fn align_down_4k(self) -> Self {
        self.align_down(PAGE_SIZE_4K)
    }

    /// Aligns the address upwards to 4096 (bytes).
    #[inline]
    pub fn align_up_4k(self) -> Self {
        self.align_up(PAGE_SIZE_4K)
    }

    /// Returns the offset of the address within a 4K-sized page.
    #[inline]
    pub fn align_offset_4k(self) -> usize {
        self.align_offset(PAGE_SIZE_4K)
    }

    /// Checks whether the address is 4K-aligned.
    #[inline]
    pub fn is_aligned_4k(self) -> bool {
        self.is_aligned(PAGE_SIZE_4K)
    }
}

impl VirtAddr {
    /// Converts an `usize` to a virtual address.
    #[inline]
    pub const fn from(addr: usize) -> Self {
        Self(addr)
    }

    /// Converts the address to an `usize`.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0
    }

    /// Converts the virtual address to a raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *const u8 {
        self.0 as *const u8
    }

    /// Converts the virtual address to a mutable raw pointer.
    #[inline]
    pub const fn as_mut_ptr(self) -> *mut u8 {
        self.0 as *mut u8
    }

    /// Aligns the address downwards to the given alignment.
    ///
    /// See the [`align_down`] function for more information.
    #[inline]
    pub fn align_down<U>(self, align: U) -> Self
    where
        U: Into<usize>,
    {
        Self(align_down(self.0, align.into()))
    }

    /// Aligns the address upwards to the given alignment.
    ///
    /// See the [`align_up`] function for more information.
    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
    where
        U: Into<usize>,
    {
        Self(align_up(self.0, align.into()))
    }

    /// Returns the offset of the address within the given alignment.
    ///
    /// See the [`align_offset`] function for more information.
    #[inline]
    pub fn align_offset<U>(self, align: U) -> usize
    where
        U: Into<usize>,
    {
        align_offset(self.0, align.into())
    }

    /// Checks whether the address has the demanded alignment.
    ///
    /// See the [`is_aligned`] function for more information.
    #[inline]
    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<usize>,
    {
        is_aligned(self.0, align.into())
    }

    /// Aligns the address downwards to 4096 (bytes).
    #[inline]
    pub fn align_down_4k(self) -> Self {
        self.align_down(PAGE_SIZE_4K)
    }

    /// Aligns the address upwards to 4096 (bytes).
    #[inline]
    pub fn align_up_4k(self) -> Self {
        self.align_up(PAGE_SIZE_4K)
    }

    /// Returns the offset of the address within a 4K-sized page.
    #[inline]
    pub fn align_offset_4k(self) -> usize {
        self.align_offset(PAGE_SIZE_4K)
    }

    /// Checks whether the address is 4K-aligned.
    #[inline]
    pub fn is_aligned_4k(self) -> bool {
        self.is_aligned(PAGE_SIZE_4K)
    }
}

impl From<usize> for PhysAddr {
    #[inline]
    fn from(addr: usize) -> Self {
        Self(addr)
    }
}

impl From<usize> for VirtAddr {
    #[inline]
    fn from(addr: usize) -> Self {
        Self(addr)
    }
}

impl From<PhysAddr> for usize {
    #[inline]
    fn from(addr: PhysAddr) -> usize {
        addr.0
    }
}

impl From<VirtAddr> for usize {
    #[inline]
    fn from(addr: VirtAddr) -> usize {
        addr.0
    }
}

impl Add<usize> for PhysAddr {
    type Output = Self;
    #[inline]
    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for PhysAddr {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl Sub<usize> for PhysAddr {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: usize) -> Self {
        Self(self.0 - rhs)
    }
}

impl SubAssign<usize> for PhysAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}

impl Add<usize> for VirtAddr {
    type Output = Self;
    #[inline]
    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for VirtAddr {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl Sub<usize> for VirtAddr {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: usize) -> Self {
        Self(self.0 - rhs)
    }
}

impl SubAssign<usize> for VirtAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}

impl fmt::LowerHex for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}

impl fmt::UpperHex for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#X}", self.0))
    }
}

impl fmt::LowerHex for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}

impl fmt::UpperHex for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#X}", self.0))
    }
}
