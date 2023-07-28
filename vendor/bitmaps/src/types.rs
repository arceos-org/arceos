// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use core::fmt::Debug;

/// A trait that defines generalised operations on a `Bits::Store` type.
pub trait BitOps {
    fn get(bits: &Self, index: usize) -> bool;
    fn set(bits: &mut Self, index: usize, value: bool) -> bool;
    fn len(bits: &Self) -> usize;
    fn first_index(bits: &Self) -> Option<usize>;
    fn first_false_index(bits: &Self) -> Option<usize>;
    fn last_index(bits: &Self) -> Option<usize>;
    fn last_false_index(bits: &Self) -> Option<usize>;
    fn next_index(bits: &Self, index: usize) -> Option<usize>;
    fn next_false_index(bits: &Self, index: usize) -> Option<usize>;
    fn prev_index(bits: &Self, index: usize) -> Option<usize>;
    fn prev_false_index(bits: &Self, index: usize) -> Option<usize>;
    fn bit_and(bits: &mut Self, other_bits: &Self);
    fn bit_or(bits: &mut Self, other_bits: &Self);
    fn bit_xor(bits: &mut Self, other_bits: &Self);
    fn invert(bits: &mut Self);
    fn make_mask(shift: usize) -> Self;
    fn bit_size() -> usize;
    #[cfg(feature = "std")]
    fn to_hex(bits: &Self) -> String;
}

impl BitOps for bool {
    #[inline]
    fn get(bits: &Self, index: usize) -> bool {
        debug_assert!(index == 0);
        *bits
    }

    #[inline]
    fn set(bits: &mut Self, index: usize, value: bool) -> bool {
        debug_assert!(index == 0);
        core::mem::replace(bits, value)
    }

    #[inline]
    fn len(bits: &Self) -> usize {
        if *bits {
            1
        } else {
            0
        }
    }

    #[inline]
    fn first_index(bits: &Self) -> Option<usize> {
        if *bits {
            Some(0)
        } else {
            None
        }
    }

    #[inline]
    fn first_false_index(bits: &Self) -> Option<usize> {
        if !*bits {
            Some(0)
        } else {
            None
        }
    }

    #[inline]
    fn last_index(bits: &Self) -> Option<usize> {
        if *bits {
            Some(0)
        } else {
            None
        }
    }

    #[inline]
    fn last_false_index(bits: &Self) -> Option<usize> {
        if !*bits {
            Some(0)
        } else {
            None
        }
    }
    #[inline]
    fn next_index(bits: &Self, index: usize) -> Option<usize> {
        debug_assert!(index == 0);
        if *bits {
            None
        } else {
            None
        }
    }

    #[inline]
    fn next_false_index(bits: &Self, index: usize) -> Option<usize> {
        debug_assert!(index == 0);
        if *bits {
            None
        } else {
            None
        }
    }

    #[inline]
    fn prev_index(bits: &Self, index: usize) -> Option<usize> {
        debug_assert!(index == 0);
        if *bits {
            None
        } else {
            None
        }
    }

    #[inline]
    fn prev_false_index(bits: &Self, index: usize) -> Option<usize> {
        debug_assert!(index == 0);
        if *bits {
            None
        } else {
            None
        }
    }

    #[inline]
    fn bit_and(bits: &mut Self, other_bits: &Self) {
        *bits &= *other_bits;
    }

    #[inline]
    fn bit_or(bits: &mut Self, other_bits: &Self) {
        *bits |= *other_bits;
    }

    #[inline]
    fn bit_xor(bits: &mut Self, other_bits: &Self) {
        *bits ^= *other_bits;
    }

    #[inline]
    fn invert(bits: &mut Self) {
        *bits = !*bits;
    }

    #[inline]
    fn make_mask(shift: usize) -> Self {
        shift > 0
    }

    #[cfg(feature = "std")]
    fn to_hex(bits: &Self) -> String {
        if *bits {
            "1".to_owned()
        } else {
            "0".to_owned()
        }
    }

    fn bit_size() -> usize {
        1
    }
}

macro_rules! bitops_for {
    ($target:ty) => {
        impl BitOps for $target {
            #[inline]
            fn get(bits: &Self, index: usize) -> bool {
                bits & (1 << index) != 0
            }

            #[inline]
            fn set(bits: &mut Self, index: usize, value: bool) -> bool {
                let mask = 1 << index;
                let prev = *bits & mask;
                if value {
                    *bits |= mask;
                } else {
                    *bits &= !mask;
                }
                prev != 0
            }

            #[inline]
            fn len(bits: &Self) -> usize {
                bits.count_ones() as usize
            }

            #[inline]
            fn first_index(bits: &Self) -> Option<usize> {
                if *bits == 0 {
                    None
                } else {
                    Some(bits.trailing_zeros() as usize)
                }
            }

            #[inline]
            fn first_false_index(bits: &Self) -> Option<usize> {
                if *bits == <$target>::MAX {
                    None
                } else {
                    Some(bits.trailing_ones() as usize)
                }
            }

            #[inline]
            fn last_index(bits: &Self) -> Option<usize> {
                if *bits == 0 {
                    None
                } else {
                    Some(<$target>::BITS as usize - 1 - (bits.leading_zeros() as usize))
                }
            }

            #[inline]
            fn last_false_index(bits: &Self) -> Option<usize> {
                if *bits == <$target>::MAX {
                    None
                } else {
                    Some(<$target>::BITS as usize - 1 - bits.leading_ones() as usize)
                }
            }

            #[inline]
            fn next_index(bits: &Self, index: usize) -> Option<usize> {
                if *bits == 0 || index >= <$target>::BITS as usize - 1 {
                    None
                } else {
                    let intermediate = (*bits & (<$target>::MAX.overflowing_shl(1 + index as u32).0));

                    if intermediate == 0 {
                        None
                    } else {
                        Some(intermediate.trailing_zeros() as usize)
                    }
                }
            }

            #[inline]
            fn next_false_index(bits: &Self, index: usize) -> Option<usize> {
                if *bits == <$target>::MAX || index >= <$target>::BITS as usize - 1 {
                    None
                } else {
                    let intermediate = (*bits | ((1 << (index + 1)) - 1));

                    if intermediate == <$target>::MAX {
                        None
                    } else {
                        Some(intermediate.trailing_ones() as usize)
                    }
                }
            }

            #[inline]
            fn prev_index(bits: &Self, index: usize) -> Option<usize> {
                if *bits == 0 || index == 0 {
                    None
                } else {
                    let intermediate = bits & ((1 << index) - 1);

                    if intermediate == 0 {
                        None
                    } else {
                        Some(<$target>::BITS as usize - 1 - (intermediate.leading_zeros() as usize))
                    }
                }
            }

            #[inline]
            fn prev_false_index(bits: &Self, index: usize) -> Option<usize> {
                if *bits == <$target>::MAX || index == 0 {
                    None
                } else {
                    let intermediate = bits | (<$target>::MAX.overflowing_shl(index as u32 - 1).0);

                    if intermediate == <$target>::MAX {
                        None
                    } else {
                        Some(<$target>::BITS as usize - 1 - (intermediate.leading_zeros() as usize))
                    }
                }
            }

            #[inline]
            fn bit_and(bits: &mut Self, other_bits: &Self) {
                *bits &= *other_bits;
            }

            #[inline]
            fn bit_or(bits: &mut Self, other_bits: &Self) {
                *bits |= *other_bits;
            }

            #[inline]
            fn bit_xor(bits: &mut Self, other_bits: &Self) {
                *bits ^= *other_bits;
            }

            #[inline]
            fn invert(bits: &mut Self) {
                *bits = !*bits;
            }

            #[inline]
            fn make_mask(shift: usize) -> Self {
                (1 << shift) - 1
            }

            #[cfg(feature = "std")]
            fn to_hex(bits: &Self) -> String {
                format!("{:x}", bits)
            }

            #[inline]
            fn bit_size() -> usize {
                <$target>::BITS as usize
            }
        }
    };
}

macro_rules! bitops_for_big {
    ($words:expr) => {
        impl BitOps for [u128; $words] {
            #[inline]
            fn get(bits: &Self, index: usize) -> bool {
                let word_index = index / 128;
                let index = index & 127;
                bits[word_index] & (1 << index) != 0
            }

            #[inline]
            fn set(bits: &mut Self, index: usize, value: bool) -> bool {
                let word_index = index / 128;
                let index = index & 127;

                let mask = 1 << (index & 127);
                let bits = &mut bits[word_index];
                let prev = *bits & mask;
                if value {
                    *bits |= mask;
                } else {
                    *bits &= !mask;
                }
                prev != 0
            }

            fn make_mask(shift: usize) -> Self {
                let word_index = shift / 128;
                let index = shift & 127;
                let mut out = [0; $words];
                for (chunk_index, chunk) in out.iter_mut().enumerate() {
                    if chunk_index < word_index {
                        *chunk = !0u128;
                    } else if chunk_index == word_index {
                        *chunk = (1 << index) - 1;
                    } else {
                        return out;
                    }
                }
                out
            }

            #[inline]
            fn len(bits: &Self) -> usize {
                bits.iter().fold(0, |acc, next| acc + next.count_ones()) as usize
            }

            #[inline]
            fn first_index(bits: &Self) -> Option<usize> {
                for (index, part) in bits.iter().enumerate() {
                    if *part != 0u128 {
                        return Some(part.trailing_zeros() as usize + (128 * index));
                    }
                }
                None
            }

            #[inline]
            fn last_index(bits: &Self) -> Option<usize> {
                for (index, part) in bits.iter().enumerate().rev() {
                    if *part != 0u128 {
                        return Some((128 * index) + <u128 as BitOps>::last_index(part).unwrap());
                    }
                }
                None
            }

            #[inline]
            fn next_index(bits: &Self, index: usize) -> Option<usize> {
                let segment: usize = index / 128;

                if (segment >= bits.len()) {
                    return None
                }

                let intermediate = <u128 as BitOps>::next_index(&bits[segment], index % 128);

                if let Some(index) = intermediate {
                    Some(segment * 128 + index)
                } else {
                    for (index, part) in bits[(segment + 1)..].iter().enumerate() {
                        if *part != 0u128 {
                            return Some(128 * (segment + 1 + index) + <u128 as BitOps>::first_index(part).unwrap());
                        }
                    }

                    None
                }
            }

            #[inline]
            fn prev_index(bits: &Self, index: usize) -> Option<usize> {
                let segment: usize = index / 128;

                if (segment >= bits.len()) {
                    return None
                }

                let intermediate = <u128 as BitOps>::prev_index(&bits[segment], index % 128);

                if let Some(index) = intermediate {
                    Some(segment * 128 + index)
                } else {
                    for (index, part) in bits[..segment].iter().enumerate().rev() {
                        if *part != 0u128 {
                            return Some(<u128 as BitOps>::last_index(part).unwrap() + (128 * index));
                        }
                    }
                    None
                }
            }

            #[inline]
            fn first_false_index(bits: &Self) -> Option<usize> {
                for (index, part) in bits.iter().enumerate() {
                    if *part != u128::MAX {
                        return Some(part.trailing_ones() as usize + (128 * index));
                    }
                }
                None
            }

            #[inline]
            fn last_false_index(bits: &Self) -> Option<usize> {
                for (index, part) in bits.iter().enumerate().rev() {
                    if *part != u128::MAX {
                        return Some(<u128 as BitOps>::last_false_index(part).unwrap() + (128 * index));
                    }
                }
                None
            }

            #[inline]
            fn next_false_index(bits: &Self, index: usize) -> Option<usize> {
                let segment: usize = index / 128;

                if (segment >= bits.len()) {
                    return None
                }

                let intermediate = <u128 as BitOps>::next_false_index(&bits[segment], index % 128);

                if let Some(index) = intermediate {
                    Some(segment * 128 + index)
                } else {
                    for (index, part) in bits[(segment + 1)..].iter().enumerate() {
                        if *part != u128::MAX {
                            return Some(128 * (segment + 1 + index) + <u128 as BitOps>::first_false_index(part).unwrap());
                        }
                    }

                    None
                }
            }

            #[inline]
            fn prev_false_index(bits: &Self, index: usize) -> Option<usize> {
                let segment: usize = index / 128;

                if (segment >= bits.len()) {
                    return None
                }

                let intermediate = <u128 as BitOps>::prev_false_index(&bits[segment], index % 128);

                if let Some(index) = intermediate {
                    Some(segment * 128 + index)
                } else {
                    for (index, part) in bits[..segment].iter().enumerate().rev() {
                        if *part != u128::MAX {
                            return Some(<u128 as BitOps>::last_false_index(part).unwrap() + (128 * index));
                        }
                    }

                    None
                }
            }

            #[inline]
            fn bit_and(bits: &mut Self, other_bits: &Self) {
                for (left, right) in bits.iter_mut().zip(other_bits.iter()) {
                    *left &= *right;
                }
            }

            #[inline]
            fn bit_or(bits: &mut Self, other_bits: &Self) {
                for (left, right) in bits.iter_mut().zip(other_bits.iter()) {
                    *left |= *right;
                }
            }

            #[inline]
            fn bit_xor(bits: &mut Self, other_bits: &Self) {
                for (left, right) in bits.iter_mut().zip(other_bits.iter()) {
                    *left ^= *right;
                }
            }

            #[inline]
            fn invert(bits: &mut Self) {
                for chunk in bits.iter_mut() {
                    *chunk = !*chunk;
                }
            }

            #[cfg(feature = "std")]
            fn to_hex(bits: &Self) -> String {
                let mut out = String::new();
                for chunk in bits {
                    out += &format!("{:x}", chunk);
                }
                out
            }

            #[inline]
            fn bit_size() -> usize {
                (<u128>::BITS * $words) as usize
            }
        }
    };
}

bitops_for!(u8);
bitops_for!(u16);
bitops_for!(u32);
bitops_for!(u64);
bitops_for!(u128);

bitops_for_big!(2);
bitops_for_big!(3);
bitops_for_big!(4);
bitops_for_big!(5);
bitops_for_big!(6);
bitops_for_big!(7);
bitops_for_big!(8);

/// A type level number signifying the number of bits in a bitmap.
///
/// This trait is implemented for type level numbers from `U1` to `U1024`.
///
/// # Examples
///
/// ```rust
/// # #[macro_use] extern crate bitmaps;
/// # use bitmaps::{Bits, BitsImpl};
/// assert_eq!(
///     std::mem::size_of::<<BitsImpl<10> as Bits>::Store>(),
///     std::mem::size_of::<u16>()
/// );
/// ```
pub trait Bits {
    /// The number of bits
    const VALUE: usize;
    /// A primitive integer type suitable for storing this many bits.
    type Store: BitOps + Default + Copy + PartialEq + Debug;

    /// The underlying data type might have some trailing bits, which would
    /// result in an invalid value being returned.
    ///
    /// Thankfully, this only happens for 'false_index'-functions (bits higher
    /// than VALUE - 1 cannot be set), and even then only for functions that
    /// might seek in that area: that is all forward seeking functions, and
    /// the one seeking backwards from the end (last_false_index).
    /// prev_false_index is not affected, because the supplied index must be
    /// valid, and any index lower than that is also valid.
    #[inline]
    fn corrected_first_false_index(store: &Self::Store) -> Option<usize> {
        let result = Self::Store::first_false_index(store);

        if Self::Store::bit_size() == Self::VALUE {
            result
        } else if let Some(result) = result {
            if result >= Self::VALUE {
                None
            } else {
                Some(result)
            }
        } else {
            None
        }
    }

    /// The underlying data type might have some trailing bits, which would
    /// result in an invalid result, so we check against that here.
    #[inline]
    fn corrected_last_false_index(store: &Self::Store) -> Option<usize> {
        if Self::Store::bit_size() == Self::VALUE {
            Self::Store::last_false_index(store)
        } else {
            Self::Store::prev_false_index(store, Self::VALUE)
        }
    }

    #[inline]
    fn corrected_next_false_index(store: &Self::Store, index: usize) -> Option<usize> {
        let result = Self::Store::next_false_index(store, index);

        if Self::Store::bit_size() == Self::VALUE {
            result
        } else if let Some(result) = result {
            if result >= Self::VALUE {
                None
            } else {
                Some(result)
            }
        } else {
            None
        }
    }
}

#[doc(hidden)]
pub struct BitsImpl<const N: usize>;

impl Bits for BitsImpl<1> {
    const VALUE: usize = 1;
    type Store = bool;
}

macro_rules! bits_for {
    ($num:expr, $result:ty) => {
        impl Bits for BitsImpl<$num> {
            const VALUE: usize = $num;
            type Store = $result;
        }
    };
}

macro_rules! bits_for_big {
    ($num:expr, $words:expr) => {
        impl Bits for BitsImpl<$num> {
            const VALUE: usize = $num;
            type Store = [u128; $words];
        }
    };
}

bits_for!(2, u8);
bits_for!(3, u8);
bits_for!(4, u8);
bits_for!(5, u8);
bits_for!(6, u8);
bits_for!(7, u8);
bits_for!(8, u8);
bits_for!(9, u16);
bits_for!(10, u16);
bits_for!(11, u16);
bits_for!(12, u16);
bits_for!(13, u16);
bits_for!(14, u16);
bits_for!(15, u16);
bits_for!(16, u16);
bits_for!(17, u32);
bits_for!(18, u32);
bits_for!(19, u32);
bits_for!(20, u32);
bits_for!(21, u32);
bits_for!(22, u32);
bits_for!(23, u32);
bits_for!(24, u32);
bits_for!(25, u32);
bits_for!(26, u32);
bits_for!(27, u32);
bits_for!(28, u32);
bits_for!(29, u32);
bits_for!(30, u32);
bits_for!(31, u32);
bits_for!(32, u32);
bits_for!(33, u64);
bits_for!(34, u64);
bits_for!(35, u64);
bits_for!(36, u64);
bits_for!(37, u64);
bits_for!(38, u64);
bits_for!(39, u64);
bits_for!(40, u64);
bits_for!(41, u64);
bits_for!(42, u64);
bits_for!(43, u64);
bits_for!(44, u64);
bits_for!(45, u64);
bits_for!(46, u64);
bits_for!(47, u64);
bits_for!(48, u64);
bits_for!(49, u64);
bits_for!(50, u64);
bits_for!(51, u64);
bits_for!(52, u64);
bits_for!(53, u64);
bits_for!(54, u64);
bits_for!(55, u64);
bits_for!(56, u64);
bits_for!(57, u64);
bits_for!(58, u64);
bits_for!(59, u64);
bits_for!(60, u64);
bits_for!(61, u64);
bits_for!(62, u64);
bits_for!(63, u64);
bits_for!(64, u64);
bits_for!(65, u128);
bits_for!(66, u128);
bits_for!(67, u128);
bits_for!(68, u128);
bits_for!(69, u128);
bits_for!(70, u128);
bits_for!(71, u128);
bits_for!(72, u128);
bits_for!(73, u128);
bits_for!(74, u128);
bits_for!(75, u128);
bits_for!(76, u128);
bits_for!(77, u128);
bits_for!(78, u128);
bits_for!(79, u128);
bits_for!(80, u128);
bits_for!(81, u128);
bits_for!(82, u128);
bits_for!(83, u128);
bits_for!(84, u128);
bits_for!(85, u128);
bits_for!(86, u128);
bits_for!(87, u128);
bits_for!(88, u128);
bits_for!(89, u128);
bits_for!(90, u128);
bits_for!(91, u128);
bits_for!(92, u128);
bits_for!(93, u128);
bits_for!(94, u128);
bits_for!(95, u128);
bits_for!(96, u128);
bits_for!(97, u128);
bits_for!(98, u128);
bits_for!(99, u128);
bits_for!(100, u128);
bits_for!(101, u128);
bits_for!(102, u128);
bits_for!(103, u128);
bits_for!(104, u128);
bits_for!(105, u128);
bits_for!(106, u128);
bits_for!(107, u128);
bits_for!(108, u128);
bits_for!(109, u128);
bits_for!(110, u128);
bits_for!(111, u128);
bits_for!(112, u128);
bits_for!(113, u128);
bits_for!(114, u128);
bits_for!(115, u128);
bits_for!(116, u128);
bits_for!(117, u128);
bits_for!(118, u128);
bits_for!(119, u128);
bits_for!(120, u128);
bits_for!(121, u128);
bits_for!(122, u128);
bits_for!(123, u128);
bits_for!(124, u128);
bits_for!(125, u128);
bits_for!(126, u128);
bits_for!(127, u128);
bits_for!(128, u128);

bits_for_big!(129, 2);
bits_for_big!(130, 2);
bits_for_big!(131, 2);
bits_for_big!(132, 2);
bits_for_big!(133, 2);
bits_for_big!(134, 2);
bits_for_big!(135, 2);
bits_for_big!(136, 2);
bits_for_big!(137, 2);
bits_for_big!(138, 2);
bits_for_big!(139, 2);
bits_for_big!(140, 2);
bits_for_big!(141, 2);
bits_for_big!(142, 2);
bits_for_big!(143, 2);
bits_for_big!(144, 2);
bits_for_big!(145, 2);
bits_for_big!(146, 2);
bits_for_big!(147, 2);
bits_for_big!(148, 2);
bits_for_big!(149, 2);
bits_for_big!(150, 2);
bits_for_big!(151, 2);
bits_for_big!(152, 2);
bits_for_big!(153, 2);
bits_for_big!(154, 2);
bits_for_big!(155, 2);
bits_for_big!(156, 2);
bits_for_big!(157, 2);
bits_for_big!(158, 2);
bits_for_big!(159, 2);
bits_for_big!(160, 2);
bits_for_big!(161, 2);
bits_for_big!(162, 2);
bits_for_big!(163, 2);
bits_for_big!(164, 2);
bits_for_big!(165, 2);
bits_for_big!(166, 2);
bits_for_big!(167, 2);
bits_for_big!(168, 2);
bits_for_big!(169, 2);
bits_for_big!(170, 2);
bits_for_big!(171, 2);
bits_for_big!(172, 2);
bits_for_big!(173, 2);
bits_for_big!(174, 2);
bits_for_big!(175, 2);
bits_for_big!(176, 2);
bits_for_big!(177, 2);
bits_for_big!(178, 2);
bits_for_big!(179, 2);
bits_for_big!(180, 2);
bits_for_big!(181, 2);
bits_for_big!(182, 2);
bits_for_big!(183, 2);
bits_for_big!(184, 2);
bits_for_big!(185, 2);
bits_for_big!(186, 2);
bits_for_big!(187, 2);
bits_for_big!(188, 2);
bits_for_big!(189, 2);
bits_for_big!(190, 2);
bits_for_big!(191, 2);
bits_for_big!(192, 2);
bits_for_big!(193, 2);
bits_for_big!(194, 2);
bits_for_big!(195, 2);
bits_for_big!(196, 2);
bits_for_big!(197, 2);
bits_for_big!(198, 2);
bits_for_big!(199, 2);
bits_for_big!(200, 2);
bits_for_big!(201, 2);
bits_for_big!(202, 2);
bits_for_big!(203, 2);
bits_for_big!(204, 2);
bits_for_big!(205, 2);
bits_for_big!(206, 2);
bits_for_big!(207, 2);
bits_for_big!(208, 2);
bits_for_big!(209, 2);
bits_for_big!(210, 2);
bits_for_big!(211, 2);
bits_for_big!(212, 2);
bits_for_big!(213, 2);
bits_for_big!(214, 2);
bits_for_big!(215, 2);
bits_for_big!(216, 2);
bits_for_big!(217, 2);
bits_for_big!(218, 2);
bits_for_big!(219, 2);
bits_for_big!(220, 2);
bits_for_big!(221, 2);
bits_for_big!(222, 2);
bits_for_big!(223, 2);
bits_for_big!(224, 2);
bits_for_big!(225, 2);
bits_for_big!(226, 2);
bits_for_big!(227, 2);
bits_for_big!(228, 2);
bits_for_big!(229, 2);
bits_for_big!(230, 2);
bits_for_big!(231, 2);
bits_for_big!(232, 2);
bits_for_big!(233, 2);
bits_for_big!(234, 2);
bits_for_big!(235, 2);
bits_for_big!(236, 2);
bits_for_big!(237, 2);
bits_for_big!(238, 2);
bits_for_big!(239, 2);
bits_for_big!(240, 2);
bits_for_big!(241, 2);
bits_for_big!(242, 2);
bits_for_big!(243, 2);
bits_for_big!(244, 2);
bits_for_big!(245, 2);
bits_for_big!(246, 2);
bits_for_big!(247, 2);
bits_for_big!(248, 2);
bits_for_big!(249, 2);
bits_for_big!(250, 2);
bits_for_big!(251, 2);
bits_for_big!(252, 2);
bits_for_big!(253, 2);
bits_for_big!(254, 2);
bits_for_big!(255, 2);
bits_for_big!(256, 2);

bits_for_big!(257, 3);
bits_for_big!(258, 3);
bits_for_big!(259, 3);
bits_for_big!(260, 3);
bits_for_big!(261, 3);
bits_for_big!(262, 3);
bits_for_big!(263, 3);
bits_for_big!(264, 3);
bits_for_big!(265, 3);
bits_for_big!(266, 3);
bits_for_big!(267, 3);
bits_for_big!(268, 3);
bits_for_big!(269, 3);
bits_for_big!(270, 3);
bits_for_big!(271, 3);
bits_for_big!(272, 3);
bits_for_big!(273, 3);
bits_for_big!(274, 3);
bits_for_big!(275, 3);
bits_for_big!(276, 3);
bits_for_big!(277, 3);
bits_for_big!(278, 3);
bits_for_big!(279, 3);
bits_for_big!(280, 3);
bits_for_big!(281, 3);
bits_for_big!(282, 3);
bits_for_big!(283, 3);
bits_for_big!(284, 3);
bits_for_big!(285, 3);
bits_for_big!(286, 3);
bits_for_big!(287, 3);
bits_for_big!(288, 3);
bits_for_big!(289, 3);
bits_for_big!(290, 3);
bits_for_big!(291, 3);
bits_for_big!(292, 3);
bits_for_big!(293, 3);
bits_for_big!(294, 3);
bits_for_big!(295, 3);
bits_for_big!(296, 3);
bits_for_big!(297, 3);
bits_for_big!(298, 3);
bits_for_big!(299, 3);
bits_for_big!(300, 3);
bits_for_big!(301, 3);
bits_for_big!(302, 3);
bits_for_big!(303, 3);
bits_for_big!(304, 3);
bits_for_big!(305, 3);
bits_for_big!(306, 3);
bits_for_big!(307, 3);
bits_for_big!(308, 3);
bits_for_big!(309, 3);
bits_for_big!(310, 3);
bits_for_big!(311, 3);
bits_for_big!(312, 3);
bits_for_big!(313, 3);
bits_for_big!(314, 3);
bits_for_big!(315, 3);
bits_for_big!(316, 3);
bits_for_big!(317, 3);
bits_for_big!(318, 3);
bits_for_big!(319, 3);
bits_for_big!(320, 3);
bits_for_big!(321, 3);
bits_for_big!(322, 3);
bits_for_big!(323, 3);
bits_for_big!(324, 3);
bits_for_big!(325, 3);
bits_for_big!(326, 3);
bits_for_big!(327, 3);
bits_for_big!(328, 3);
bits_for_big!(329, 3);
bits_for_big!(330, 3);
bits_for_big!(331, 3);
bits_for_big!(332, 3);
bits_for_big!(333, 3);
bits_for_big!(334, 3);
bits_for_big!(335, 3);
bits_for_big!(336, 3);
bits_for_big!(337, 3);
bits_for_big!(338, 3);
bits_for_big!(339, 3);
bits_for_big!(340, 3);
bits_for_big!(341, 3);
bits_for_big!(342, 3);
bits_for_big!(343, 3);
bits_for_big!(344, 3);
bits_for_big!(345, 3);
bits_for_big!(346, 3);
bits_for_big!(347, 3);
bits_for_big!(348, 3);
bits_for_big!(349, 3);
bits_for_big!(350, 3);
bits_for_big!(351, 3);
bits_for_big!(352, 3);
bits_for_big!(353, 3);
bits_for_big!(354, 3);
bits_for_big!(355, 3);
bits_for_big!(356, 3);
bits_for_big!(357, 3);
bits_for_big!(358, 3);
bits_for_big!(359, 3);
bits_for_big!(360, 3);
bits_for_big!(361, 3);
bits_for_big!(362, 3);
bits_for_big!(363, 3);
bits_for_big!(364, 3);
bits_for_big!(365, 3);
bits_for_big!(366, 3);
bits_for_big!(367, 3);
bits_for_big!(368, 3);
bits_for_big!(369, 3);
bits_for_big!(370, 3);
bits_for_big!(371, 3);
bits_for_big!(372, 3);
bits_for_big!(373, 3);
bits_for_big!(374, 3);
bits_for_big!(375, 3);
bits_for_big!(376, 3);
bits_for_big!(377, 3);
bits_for_big!(378, 3);
bits_for_big!(379, 3);
bits_for_big!(380, 3);
bits_for_big!(381, 3);
bits_for_big!(382, 3);
bits_for_big!(383, 3);
bits_for_big!(384, 3);

bits_for_big!(385, 4);
bits_for_big!(386, 4);
bits_for_big!(387, 4);
bits_for_big!(388, 4);
bits_for_big!(389, 4);
bits_for_big!(390, 4);
bits_for_big!(391, 4);
bits_for_big!(392, 4);
bits_for_big!(393, 4);
bits_for_big!(394, 4);
bits_for_big!(395, 4);
bits_for_big!(396, 4);
bits_for_big!(397, 4);
bits_for_big!(398, 4);
bits_for_big!(399, 4);
bits_for_big!(400, 4);
bits_for_big!(401, 4);
bits_for_big!(402, 4);
bits_for_big!(403, 4);
bits_for_big!(404, 4);
bits_for_big!(405, 4);
bits_for_big!(406, 4);
bits_for_big!(407, 4);
bits_for_big!(408, 4);
bits_for_big!(409, 4);
bits_for_big!(410, 4);
bits_for_big!(411, 4);
bits_for_big!(412, 4);
bits_for_big!(413, 4);
bits_for_big!(414, 4);
bits_for_big!(415, 4);
bits_for_big!(416, 4);
bits_for_big!(417, 4);
bits_for_big!(418, 4);
bits_for_big!(419, 4);
bits_for_big!(420, 4);
bits_for_big!(421, 4);
bits_for_big!(422, 4);
bits_for_big!(423, 4);
bits_for_big!(424, 4);
bits_for_big!(425, 4);
bits_for_big!(426, 4);
bits_for_big!(427, 4);
bits_for_big!(428, 4);
bits_for_big!(429, 4);
bits_for_big!(430, 4);
bits_for_big!(431, 4);
bits_for_big!(432, 4);
bits_for_big!(433, 4);
bits_for_big!(434, 4);
bits_for_big!(435, 4);
bits_for_big!(436, 4);
bits_for_big!(437, 4);
bits_for_big!(438, 4);
bits_for_big!(439, 4);
bits_for_big!(440, 4);
bits_for_big!(441, 4);
bits_for_big!(442, 4);
bits_for_big!(443, 4);
bits_for_big!(444, 4);
bits_for_big!(445, 4);
bits_for_big!(446, 4);
bits_for_big!(447, 4);
bits_for_big!(448, 4);
bits_for_big!(449, 4);
bits_for_big!(450, 4);
bits_for_big!(451, 4);
bits_for_big!(452, 4);
bits_for_big!(453, 4);
bits_for_big!(454, 4);
bits_for_big!(455, 4);
bits_for_big!(456, 4);
bits_for_big!(457, 4);
bits_for_big!(458, 4);
bits_for_big!(459, 4);
bits_for_big!(460, 4);
bits_for_big!(461, 4);
bits_for_big!(462, 4);
bits_for_big!(463, 4);
bits_for_big!(464, 4);
bits_for_big!(465, 4);
bits_for_big!(466, 4);
bits_for_big!(467, 4);
bits_for_big!(468, 4);
bits_for_big!(469, 4);
bits_for_big!(470, 4);
bits_for_big!(471, 4);
bits_for_big!(472, 4);
bits_for_big!(473, 4);
bits_for_big!(474, 4);
bits_for_big!(475, 4);
bits_for_big!(476, 4);
bits_for_big!(477, 4);
bits_for_big!(478, 4);
bits_for_big!(479, 4);
bits_for_big!(480, 4);
bits_for_big!(481, 4);
bits_for_big!(482, 4);
bits_for_big!(483, 4);
bits_for_big!(484, 4);
bits_for_big!(485, 4);
bits_for_big!(486, 4);
bits_for_big!(487, 4);
bits_for_big!(488, 4);
bits_for_big!(489, 4);
bits_for_big!(490, 4);
bits_for_big!(491, 4);
bits_for_big!(492, 4);
bits_for_big!(493, 4);
bits_for_big!(494, 4);
bits_for_big!(495, 4);
bits_for_big!(496, 4);
bits_for_big!(497, 4);
bits_for_big!(498, 4);
bits_for_big!(499, 4);
bits_for_big!(500, 4);
bits_for_big!(501, 4);
bits_for_big!(502, 4);
bits_for_big!(503, 4);
bits_for_big!(504, 4);
bits_for_big!(505, 4);
bits_for_big!(506, 4);
bits_for_big!(507, 4);
bits_for_big!(508, 4);
bits_for_big!(509, 4);
bits_for_big!(510, 4);
bits_for_big!(511, 4);
bits_for_big!(512, 4);

bits_for_big!(513, 5);
bits_for_big!(514, 5);
bits_for_big!(515, 5);
bits_for_big!(516, 5);
bits_for_big!(517, 5);
bits_for_big!(518, 5);
bits_for_big!(519, 5);
bits_for_big!(520, 5);
bits_for_big!(521, 5);
bits_for_big!(522, 5);
bits_for_big!(523, 5);
bits_for_big!(524, 5);
bits_for_big!(525, 5);
bits_for_big!(526, 5);
bits_for_big!(527, 5);
bits_for_big!(528, 5);
bits_for_big!(529, 5);
bits_for_big!(530, 5);
bits_for_big!(531, 5);
bits_for_big!(532, 5);
bits_for_big!(533, 5);
bits_for_big!(534, 5);
bits_for_big!(535, 5);
bits_for_big!(536, 5);
bits_for_big!(537, 5);
bits_for_big!(538, 5);
bits_for_big!(539, 5);
bits_for_big!(540, 5);
bits_for_big!(541, 5);
bits_for_big!(542, 5);
bits_for_big!(543, 5);
bits_for_big!(544, 5);
bits_for_big!(545, 5);
bits_for_big!(546, 5);
bits_for_big!(547, 5);
bits_for_big!(548, 5);
bits_for_big!(549, 5);
bits_for_big!(550, 5);
bits_for_big!(551, 5);
bits_for_big!(552, 5);
bits_for_big!(553, 5);
bits_for_big!(554, 5);
bits_for_big!(555, 5);
bits_for_big!(556, 5);
bits_for_big!(557, 5);
bits_for_big!(558, 5);
bits_for_big!(559, 5);
bits_for_big!(560, 5);
bits_for_big!(561, 5);
bits_for_big!(562, 5);
bits_for_big!(563, 5);
bits_for_big!(564, 5);
bits_for_big!(565, 5);
bits_for_big!(566, 5);
bits_for_big!(567, 5);
bits_for_big!(568, 5);
bits_for_big!(569, 5);
bits_for_big!(570, 5);
bits_for_big!(571, 5);
bits_for_big!(572, 5);
bits_for_big!(573, 5);
bits_for_big!(574, 5);
bits_for_big!(575, 5);
bits_for_big!(576, 5);
bits_for_big!(577, 5);
bits_for_big!(578, 5);
bits_for_big!(579, 5);
bits_for_big!(580, 5);
bits_for_big!(581, 5);
bits_for_big!(582, 5);
bits_for_big!(583, 5);
bits_for_big!(584, 5);
bits_for_big!(585, 5);
bits_for_big!(586, 5);
bits_for_big!(587, 5);
bits_for_big!(588, 5);
bits_for_big!(589, 5);
bits_for_big!(590, 5);
bits_for_big!(591, 5);
bits_for_big!(592, 5);
bits_for_big!(593, 5);
bits_for_big!(594, 5);
bits_for_big!(595, 5);
bits_for_big!(596, 5);
bits_for_big!(597, 5);
bits_for_big!(598, 5);
bits_for_big!(599, 5);
bits_for_big!(600, 5);
bits_for_big!(601, 5);
bits_for_big!(602, 5);
bits_for_big!(603, 5);
bits_for_big!(604, 5);
bits_for_big!(605, 5);
bits_for_big!(606, 5);
bits_for_big!(607, 5);
bits_for_big!(608, 5);
bits_for_big!(609, 5);
bits_for_big!(610, 5);
bits_for_big!(611, 5);
bits_for_big!(612, 5);
bits_for_big!(613, 5);
bits_for_big!(614, 5);
bits_for_big!(615, 5);
bits_for_big!(616, 5);
bits_for_big!(617, 5);
bits_for_big!(618, 5);
bits_for_big!(619, 5);
bits_for_big!(620, 5);
bits_for_big!(621, 5);
bits_for_big!(622, 5);
bits_for_big!(623, 5);
bits_for_big!(624, 5);
bits_for_big!(625, 5);
bits_for_big!(626, 5);
bits_for_big!(627, 5);
bits_for_big!(628, 5);
bits_for_big!(629, 5);
bits_for_big!(630, 5);
bits_for_big!(631, 5);
bits_for_big!(632, 5);
bits_for_big!(633, 5);
bits_for_big!(634, 5);
bits_for_big!(635, 5);
bits_for_big!(636, 5);
bits_for_big!(637, 5);
bits_for_big!(638, 5);
bits_for_big!(639, 5);
bits_for_big!(640, 5);

bits_for_big!(641, 6);
bits_for_big!(642, 6);
bits_for_big!(643, 6);
bits_for_big!(644, 6);
bits_for_big!(645, 6);
bits_for_big!(646, 6);
bits_for_big!(647, 6);
bits_for_big!(648, 6);
bits_for_big!(649, 6);
bits_for_big!(650, 6);
bits_for_big!(651, 6);
bits_for_big!(652, 6);
bits_for_big!(653, 6);
bits_for_big!(654, 6);
bits_for_big!(655, 6);
bits_for_big!(656, 6);
bits_for_big!(657, 6);
bits_for_big!(658, 6);
bits_for_big!(659, 6);
bits_for_big!(660, 6);
bits_for_big!(661, 6);
bits_for_big!(662, 6);
bits_for_big!(663, 6);
bits_for_big!(664, 6);
bits_for_big!(665, 6);
bits_for_big!(666, 6);
bits_for_big!(667, 6);
bits_for_big!(668, 6);
bits_for_big!(669, 6);
bits_for_big!(670, 6);
bits_for_big!(671, 6);
bits_for_big!(672, 6);
bits_for_big!(673, 6);
bits_for_big!(674, 6);
bits_for_big!(675, 6);
bits_for_big!(676, 6);
bits_for_big!(677, 6);
bits_for_big!(678, 6);
bits_for_big!(679, 6);
bits_for_big!(680, 6);
bits_for_big!(681, 6);
bits_for_big!(682, 6);
bits_for_big!(683, 6);
bits_for_big!(684, 6);
bits_for_big!(685, 6);
bits_for_big!(686, 6);
bits_for_big!(687, 6);
bits_for_big!(688, 6);
bits_for_big!(689, 6);
bits_for_big!(690, 6);
bits_for_big!(691, 6);
bits_for_big!(692, 6);
bits_for_big!(693, 6);
bits_for_big!(694, 6);
bits_for_big!(695, 6);
bits_for_big!(696, 6);
bits_for_big!(697, 6);
bits_for_big!(698, 6);
bits_for_big!(699, 6);
bits_for_big!(700, 6);
bits_for_big!(701, 6);
bits_for_big!(702, 6);
bits_for_big!(703, 6);
bits_for_big!(704, 6);
bits_for_big!(705, 6);
bits_for_big!(706, 6);
bits_for_big!(707, 6);
bits_for_big!(708, 6);
bits_for_big!(709, 6);
bits_for_big!(710, 6);
bits_for_big!(711, 6);
bits_for_big!(712, 6);
bits_for_big!(713, 6);
bits_for_big!(714, 6);
bits_for_big!(715, 6);
bits_for_big!(716, 6);
bits_for_big!(717, 6);
bits_for_big!(718, 6);
bits_for_big!(719, 6);
bits_for_big!(720, 6);
bits_for_big!(721, 6);
bits_for_big!(722, 6);
bits_for_big!(723, 6);
bits_for_big!(724, 6);
bits_for_big!(725, 6);
bits_for_big!(726, 6);
bits_for_big!(727, 6);
bits_for_big!(728, 6);
bits_for_big!(729, 6);
bits_for_big!(730, 6);
bits_for_big!(731, 6);
bits_for_big!(732, 6);
bits_for_big!(733, 6);
bits_for_big!(734, 6);
bits_for_big!(735, 6);
bits_for_big!(736, 6);
bits_for_big!(737, 6);
bits_for_big!(738, 6);
bits_for_big!(739, 6);
bits_for_big!(740, 6);
bits_for_big!(741, 6);
bits_for_big!(742, 6);
bits_for_big!(743, 6);
bits_for_big!(744, 6);
bits_for_big!(745, 6);
bits_for_big!(746, 6);
bits_for_big!(747, 6);
bits_for_big!(748, 6);
bits_for_big!(749, 6);
bits_for_big!(750, 6);
bits_for_big!(751, 6);
bits_for_big!(752, 6);
bits_for_big!(753, 6);
bits_for_big!(754, 6);
bits_for_big!(755, 6);
bits_for_big!(756, 6);
bits_for_big!(757, 6);
bits_for_big!(758, 6);
bits_for_big!(759, 6);
bits_for_big!(760, 6);
bits_for_big!(761, 6);
bits_for_big!(762, 6);
bits_for_big!(763, 6);
bits_for_big!(764, 6);
bits_for_big!(765, 6);
bits_for_big!(766, 6);
bits_for_big!(767, 6);
bits_for_big!(768, 6);

bits_for_big!(769, 7);
bits_for_big!(770, 7);
bits_for_big!(771, 7);
bits_for_big!(772, 7);
bits_for_big!(773, 7);
bits_for_big!(774, 7);
bits_for_big!(775, 7);
bits_for_big!(776, 7);
bits_for_big!(777, 7);
bits_for_big!(778, 7);
bits_for_big!(779, 7);
bits_for_big!(780, 7);
bits_for_big!(781, 7);
bits_for_big!(782, 7);
bits_for_big!(783, 7);
bits_for_big!(784, 7);
bits_for_big!(785, 7);
bits_for_big!(786, 7);
bits_for_big!(787, 7);
bits_for_big!(788, 7);
bits_for_big!(789, 7);
bits_for_big!(790, 7);
bits_for_big!(791, 7);
bits_for_big!(792, 7);
bits_for_big!(793, 7);
bits_for_big!(794, 7);
bits_for_big!(795, 7);
bits_for_big!(796, 7);
bits_for_big!(797, 7);
bits_for_big!(798, 7);
bits_for_big!(799, 7);
bits_for_big!(800, 7);
bits_for_big!(801, 7);
bits_for_big!(802, 7);
bits_for_big!(803, 7);
bits_for_big!(804, 7);
bits_for_big!(805, 7);
bits_for_big!(806, 7);
bits_for_big!(807, 7);
bits_for_big!(808, 7);
bits_for_big!(809, 7);
bits_for_big!(810, 7);
bits_for_big!(811, 7);
bits_for_big!(812, 7);
bits_for_big!(813, 7);
bits_for_big!(814, 7);
bits_for_big!(815, 7);
bits_for_big!(816, 7);
bits_for_big!(817, 7);
bits_for_big!(818, 7);
bits_for_big!(819, 7);
bits_for_big!(820, 7);
bits_for_big!(821, 7);
bits_for_big!(822, 7);
bits_for_big!(823, 7);
bits_for_big!(824, 7);
bits_for_big!(825, 7);
bits_for_big!(826, 7);
bits_for_big!(827, 7);
bits_for_big!(828, 7);
bits_for_big!(829, 7);
bits_for_big!(830, 7);
bits_for_big!(831, 7);
bits_for_big!(832, 7);
bits_for_big!(833, 7);
bits_for_big!(834, 7);
bits_for_big!(835, 7);
bits_for_big!(836, 7);
bits_for_big!(837, 7);
bits_for_big!(838, 7);
bits_for_big!(839, 7);
bits_for_big!(840, 7);
bits_for_big!(841, 7);
bits_for_big!(842, 7);
bits_for_big!(843, 7);
bits_for_big!(844, 7);
bits_for_big!(845, 7);
bits_for_big!(846, 7);
bits_for_big!(847, 7);
bits_for_big!(848, 7);
bits_for_big!(849, 7);
bits_for_big!(850, 7);
bits_for_big!(851, 7);
bits_for_big!(852, 7);
bits_for_big!(853, 7);
bits_for_big!(854, 7);
bits_for_big!(855, 7);
bits_for_big!(856, 7);
bits_for_big!(857, 7);
bits_for_big!(858, 7);
bits_for_big!(859, 7);
bits_for_big!(860, 7);
bits_for_big!(861, 7);
bits_for_big!(862, 7);
bits_for_big!(863, 7);
bits_for_big!(864, 7);
bits_for_big!(865, 7);
bits_for_big!(866, 7);
bits_for_big!(867, 7);
bits_for_big!(868, 7);
bits_for_big!(869, 7);
bits_for_big!(870, 7);
bits_for_big!(871, 7);
bits_for_big!(872, 7);
bits_for_big!(873, 7);
bits_for_big!(874, 7);
bits_for_big!(875, 7);
bits_for_big!(876, 7);
bits_for_big!(877, 7);
bits_for_big!(878, 7);
bits_for_big!(879, 7);
bits_for_big!(880, 7);
bits_for_big!(881, 7);
bits_for_big!(882, 7);
bits_for_big!(883, 7);
bits_for_big!(884, 7);
bits_for_big!(885, 7);
bits_for_big!(886, 7);
bits_for_big!(887, 7);
bits_for_big!(888, 7);
bits_for_big!(889, 7);
bits_for_big!(890, 7);
bits_for_big!(891, 7);
bits_for_big!(892, 7);
bits_for_big!(893, 7);
bits_for_big!(894, 7);
bits_for_big!(895, 7);
bits_for_big!(896, 7);

bits_for_big!(897, 8);
bits_for_big!(898, 8);
bits_for_big!(899, 8);
bits_for_big!(900, 8);
bits_for_big!(901, 8);
bits_for_big!(902, 8);
bits_for_big!(903, 8);
bits_for_big!(904, 8);
bits_for_big!(905, 8);
bits_for_big!(906, 8);
bits_for_big!(907, 8);
bits_for_big!(908, 8);
bits_for_big!(909, 8);
bits_for_big!(910, 8);
bits_for_big!(911, 8);
bits_for_big!(912, 8);
bits_for_big!(913, 8);
bits_for_big!(914, 8);
bits_for_big!(915, 8);
bits_for_big!(916, 8);
bits_for_big!(917, 8);
bits_for_big!(918, 8);
bits_for_big!(919, 8);
bits_for_big!(920, 8);
bits_for_big!(921, 8);
bits_for_big!(922, 8);
bits_for_big!(923, 8);
bits_for_big!(924, 8);
bits_for_big!(925, 8);
bits_for_big!(926, 8);
bits_for_big!(927, 8);
bits_for_big!(928, 8);
bits_for_big!(929, 8);
bits_for_big!(930, 8);
bits_for_big!(931, 8);
bits_for_big!(932, 8);
bits_for_big!(933, 8);
bits_for_big!(934, 8);
bits_for_big!(935, 8);
bits_for_big!(936, 8);
bits_for_big!(937, 8);
bits_for_big!(938, 8);
bits_for_big!(939, 8);
bits_for_big!(940, 8);
bits_for_big!(941, 8);
bits_for_big!(942, 8);
bits_for_big!(943, 8);
bits_for_big!(944, 8);
bits_for_big!(945, 8);
bits_for_big!(946, 8);
bits_for_big!(947, 8);
bits_for_big!(948, 8);
bits_for_big!(949, 8);
bits_for_big!(950, 8);
bits_for_big!(951, 8);
bits_for_big!(952, 8);
bits_for_big!(953, 8);
bits_for_big!(954, 8);
bits_for_big!(955, 8);
bits_for_big!(956, 8);
bits_for_big!(957, 8);
bits_for_big!(958, 8);
bits_for_big!(959, 8);
bits_for_big!(960, 8);
bits_for_big!(961, 8);
bits_for_big!(962, 8);
bits_for_big!(963, 8);
bits_for_big!(964, 8);
bits_for_big!(965, 8);
bits_for_big!(966, 8);
bits_for_big!(967, 8);
bits_for_big!(968, 8);
bits_for_big!(969, 8);
bits_for_big!(970, 8);
bits_for_big!(971, 8);
bits_for_big!(972, 8);
bits_for_big!(973, 8);
bits_for_big!(974, 8);
bits_for_big!(975, 8);
bits_for_big!(976, 8);
bits_for_big!(977, 8);
bits_for_big!(978, 8);
bits_for_big!(979, 8);
bits_for_big!(980, 8);
bits_for_big!(981, 8);
bits_for_big!(982, 8);
bits_for_big!(983, 8);
bits_for_big!(984, 8);
bits_for_big!(985, 8);
bits_for_big!(986, 8);
bits_for_big!(987, 8);
bits_for_big!(988, 8);
bits_for_big!(989, 8);
bits_for_big!(990, 8);
bits_for_big!(991, 8);
bits_for_big!(992, 8);
bits_for_big!(993, 8);
bits_for_big!(994, 8);
bits_for_big!(995, 8);
bits_for_big!(996, 8);
bits_for_big!(997, 8);
bits_for_big!(998, 8);
bits_for_big!(999, 8);
bits_for_big!(1000, 8);
bits_for_big!(1001, 8);
bits_for_big!(1002, 8);
bits_for_big!(1003, 8);
bits_for_big!(1004, 8);
bits_for_big!(1005, 8);
bits_for_big!(1006, 8);
bits_for_big!(1007, 8);
bits_for_big!(1008, 8);
bits_for_big!(1009, 8);
bits_for_big!(1010, 8);
bits_for_big!(1011, 8);
bits_for_big!(1012, 8);
bits_for_big!(1013, 8);
bits_for_big!(1014, 8);
bits_for_big!(1015, 8);
bits_for_big!(1016, 8);
bits_for_big!(1017, 8);
bits_for_big!(1018, 8);
bits_for_big!(1019, 8);
bits_for_big!(1020, 8);
bits_for_big!(1021, 8);
bits_for_big!(1022, 8);
bits_for_big!(1023, 8);
bits_for_big!(1024, 8);
