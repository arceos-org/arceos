#![no_std]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]

use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

pub const PAGE_SIZE_4K: usize = 0x1000;

#[inline]
pub const fn align_down<U>(addr: usize, align: U) -> usize
where
    U: ~const Into<usize>,
{
    addr & !(align.into() - 1)
}

#[inline]
pub const fn align_up<U>(addr: usize, align: U) -> usize
where
    U: ~const Into<usize>,
{
    let align = align.into();
    (addr + align - 1) & !(align - 1)
}

#[inline]
pub const fn align_offset<U>(addr: usize, align: U) -> usize
where
    U: ~const Into<usize>,
{
    addr & (align.into() - 1)
}

#[inline]
pub const fn is_aligned<U>(addr: usize, align: U) -> bool
where
    U: ~const Into<usize>,
{
    align_offset(addr, align) == 0
}

#[inline]
pub const fn align_down_4k(addr: usize) -> usize {
    align_down(addr, PAGE_SIZE_4K)
}

#[inline]
pub const fn align_up_4k(addr: usize) -> usize {
    align_up(addr, PAGE_SIZE_4K)
}

#[inline]
pub const fn align_offset_4k(addr: usize) -> usize {
    align_offset(addr, PAGE_SIZE_4K)
}

#[inline]
pub const fn is_aligned_4k(addr: usize) -> bool {
    is_aligned(addr, PAGE_SIZE_4K)
}

#[repr(transparent)]
#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(usize);

#[repr(transparent)]
#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(usize);

impl PhysAddr {
    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.0
    }

    #[inline]
    pub const fn align_down<U>(&self, align: U) -> Self
    where
        U: ~const Into<usize>,
    {
        Self(align_down(self.0, align))
    }

    #[inline]
    pub const fn align_up<U>(&self, align: U) -> Self
    where
        U: ~const Into<usize>,
    {
        Self(align_up(self.0, align))
    }

    #[inline]
    pub const fn align_offset<U>(&self, align: U) -> usize
    where
        U: ~const Into<usize>,
    {
        align_offset(self.0, align)
    }

    #[inline]
    pub const fn is_aligned<U>(&self, align: U) -> bool
    where
        U: ~const Into<usize>,
    {
        is_aligned(self.0, align)
    }

    #[inline]
    pub const fn align_down_4k(&self) -> Self {
        self.align_down(PAGE_SIZE_4K)
    }

    #[inline]
    pub const fn align_up_4k(&self) -> Self {
        self.align_up(PAGE_SIZE_4K)
    }

    #[inline]
    pub const fn align_offset_4k(&self) -> usize {
        self.align_offset(PAGE_SIZE_4K)
    }

    #[inline]
    pub const fn is_aligned_4k(&self) -> bool {
        self.is_aligned(PAGE_SIZE_4K)
    }
}

impl VirtAddr {
    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.0
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const u8 {
        self.0 as *const u8
    }

    #[inline]
    pub const fn as_mut_ptr(&self) -> *mut u8 {
        self.0 as *mut u8
    }

    #[inline]
    pub const fn align_down<U>(&self, align: U) -> Self
    where
        U: ~const Into<usize>,
    {
        Self(align_down(self.0, align))
    }

    #[inline]
    pub const fn align_up<U>(&self, align: U) -> Self
    where
        U: ~const Into<usize>,
    {
        Self(align_up(self.0, align))
    }

    #[inline]
    pub const fn align_offset<U>(&self, align: U) -> usize
    where
        U: ~const Into<usize>,
    {
        align_offset(self.0, align)
    }

    #[inline]
    pub const fn is_aligned<U>(&self, align: U) -> bool
    where
        U: ~const Into<usize>,
    {
        is_aligned(self.0, align)
    }

    #[inline]
    pub const fn align_down_4k(&self) -> Self {
        self.align_down(PAGE_SIZE_4K)
    }

    #[inline]
    pub const fn align_up_4k(&self) -> Self {
        self.align_up(PAGE_SIZE_4K)
    }

    #[inline]
    pub const fn align_offset_4k(&self) -> usize {
        self.align_offset(PAGE_SIZE_4K)
    }

    #[inline]
    pub const fn is_aligned_4k(&self) -> bool {
        self.is_aligned(PAGE_SIZE_4K)
    }
}

impl const From<usize> for PhysAddr {
    #[inline]
    fn from(addr: usize) -> Self {
        Self(addr)
    }
}

impl const From<usize> for VirtAddr {
    #[inline]
    fn from(addr: usize) -> Self {
        Self(addr)
    }
}

impl const From<PhysAddr> for usize {
    #[inline]
    fn from(addr: PhysAddr) -> usize {
        addr.0
    }
}

impl const From<VirtAddr> for usize {
    #[inline]
    fn from(addr: VirtAddr) -> usize {
        addr.0
    }
}

impl const Add<usize> for PhysAddr {
    type Output = Self;
    #[inline]
    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs)
    }
}

impl const AddAssign<usize> for PhysAddr {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl const Sub<usize> for PhysAddr {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: usize) -> Self {
        Self(self.0 - rhs)
    }
}

impl const SubAssign<usize> for PhysAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}

impl const Add<usize> for VirtAddr {
    type Output = Self;
    #[inline]
    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs)
    }
}

impl const AddAssign<usize> for VirtAddr {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl const Sub<usize> for VirtAddr {
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
