//! This crate adds simple bit-manipulation helpers in a convenient trait.
//! Also, it implements them for the unsigned primitive types.

#![no_std]
#![warn(missing_docs)]

#[cfg(test)]
mod tests;

use core::ops::Range;

/// A trait which provides methods for manipulating bits or bit ranges.
pub trait BitIndex {
    /// Length of the implementor type in bits.
    fn bit_length() -> usize;

    /// Obtains the value of the bit at the given position, being 0 the least
    /// significant bit.
    ///
    /// # Panics
    ///
    /// This method will panic if the index is out of bounds, e.g: `pos >=
    /// Self::bit_length()`.
    fn bit(&self, pos: usize) -> bool;

    /// Obtains the value of the bits inside the given range, being 0 the least
    /// significant bit.
    ///
    /// # Panics
    ///
    /// This method will panic if:
    ///
    ///  - Range `start` is equal or higher than `end`
    ///  - Range `end` is out of bounds, e.g: `pos.end > Self::bit_length()`
    fn bit_range(&self, pos: Range<usize>) -> Self;

    /// Sets the value of the bit at the given position, being 0 the least
    /// significant bit.
    ///
    /// # Panics
    ///
    /// This method will panic if the index is out of bounds, e.g: `pos >=
    /// Self::bit_length()`.
    fn set_bit(&mut self, pos: usize, val: bool) -> &mut Self;

    /// Sets the value of the bits inside the given range, being 0 the least
    /// significant bit.
    ///
    /// # Panics
    ///
    /// This method will panic if:
    ///
    ///  - Range `start` is equal or higher than `end`
    ///  - Range `end` is out of bounds, e.g: `pos.end > Self::bit_length()`
    ///  - Value doesn't fit in the given range
    fn set_bit_range(&mut self, pos: Range<usize>, val: Self) -> &mut Self;
}

macro_rules! bitindex_num_impl {
    ($($t:ty),*) => {$(
        impl BitIndex for $t {
            #[inline]
            fn bit_length() -> usize {
                ::core::mem::size_of::<Self>() * 8
            }

            #[inline]
            fn bit(&self, pos: usize) -> bool {
                assert!(pos < Self::bit_length());
                *self & 1 << pos != 0
            }

            #[inline]
            fn bit_range(&self, pos: Range<usize>) -> Self {
                let len = Self::bit_length();
                assert!(pos.start < pos.end && pos.end <= len);

                *self << len - pos.end >> len - pos.end + pos.start
            }

            #[inline]
            fn set_bit(&mut self, pos: usize, val: bool) -> &mut Self {
                let len = Self::bit_length();
                assert!(pos < len);

                *self ^= (Self::min_value().wrapping_sub(val as Self) ^ *self) & 1 << pos;
                self
            }

            #[inline]
            fn set_bit_range(&mut self, pos: Range<usize>, val: Self) -> &mut Self {
                let len = Self::bit_length();
                assert!(pos.start < pos.end && pos.end <= len);
                assert_eq!(val.bit_range((pos.end - pos.start)..len), 0);

                let mask = !(Self::max_value().bit_range(pos.start..pos.end) << pos.start);
                *self = *self & mask | val << pos.start;
                self
            }
        }
    )*}
}

bitindex_num_impl!(u8, u16, u32, u64, usize);
