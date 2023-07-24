//! Statically allocated arrays with guaranteed memory alignments
//!
//! # Examples
//!
//! ```
//! #![feature(const_fn)]
//!
//! use std::mem;
//!
//! use aligned::Aligned;
//!
//! // Array aligned to a 2 byte boundary
//! static X: Aligned<u16, [u8; 3]> = Aligned([0; 3]);
//!
//! // Array aligned to a 4 byte boundary
//! static Y: Aligned<u32, [u8; 3]> = Aligned([0; 3]);
//!
//! // Unaligned array
//! static Z: [u8; 3] = [0; 3];
//!
//! // You can allocate the aligned arrays on the stack too
//! let w: Aligned<u64, _> = Aligned([0u8; 3]);
//!
//! assert_eq!(mem::align_of_val(&X), 2);
//! assert_eq!(mem::align_of_val(&Y), 4);
//! assert_eq!(mem::align_of_val(&Z), 1);
//! assert_eq!(mem::align_of_val(&w), 8);
//! ```

#![deny(missing_docs)]
#![deny(warnings)]
#![cfg_attr(feature = "const-fn", feature(const_fn))]
#![no_std]

use core::{mem, ops};

/// An `ARRAY` aligned to `mem::align_of::<ALIGNMENT>()` bytes
pub struct Aligned<ALIGNMENT, ARRAY>
where
    ARRAY: ?Sized,
{
    _alignment: [ALIGNMENT; 0],
    /// The array
    pub array: ARRAY,
}

impl<T, ALIGNMENT> ops::Deref for Aligned<ALIGNMENT, [T]> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            mem::transmute(self)
        }
    }
}

impl<T, ALIGNMENT> ops::DerefMut for Aligned<ALIGNMENT, [T]> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            mem::transmute(self)
        }
    }
}

macro_rules! slice {
    ($($N:expr),+) => {
        $(
            impl<T, ALIGNMENT> ops::Deref for Aligned<ALIGNMENT, [T; $N]> {
                type Target = Aligned<ALIGNMENT, [T]>;

                fn deref(&self) -> &Self::Target {
                    unsafe {
                        mem::transmute(&self.array[..])
                    }
                }
            }

            impl<T, ALIGNMENT> ops::DerefMut for Aligned<ALIGNMENT, [T; $N]> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe {
                        mem::transmute(&mut self.array[..])
                    }
                }
            }

            impl<T, ALIGNMENT> ops::Index<ops::RangeTo<usize>>
                for Aligned<ALIGNMENT, [T; $N]>
            {
                type Output = Aligned<ALIGNMENT, [T]>;

                fn index(&self, range: ops::RangeTo<usize>) -> &Self::Output {
                    unsafe {
                        mem::transmute(self.array.index(range))
                    }
                }
            }

            impl<T, ALIGNMENT> ops::IndexMut<ops::RangeTo<usize>>
                for Aligned<ALIGNMENT, [T; $N]>
            {
                fn index_mut(
                    &mut self,
                    range: ops::RangeTo<usize>,
                ) -> &mut Self::Output {
                    unsafe {
                        mem::transmute(self.array.index_mut(range))
                    }
                }
            }
        )+
    }
}

slice!(
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
    10,
    11,
    12,
    13,
    14,
    15,
    16,
    17,
    18,
    19,
    20,
    21,
    22,
    23,
    24,
    25,
    26,
    27,
    28,
    29,
    30,
    31,
    32,
    64,
    128,
    256,
    1024
);

/// IMPLEMENTATION DETAIL
pub unsafe trait Alignment {}

/// 2 byte alignment
unsafe impl Alignment for u16 {}

/// 4 byte alignment
unsafe impl Alignment for u32 {}

/// 8 byte alignment
unsafe impl Alignment for u64 {}

/// `Aligned` constructor
#[allow(non_snake_case)]
#[cfg(feature = "const-fn")]
pub const fn Aligned<ALIGNMENT, ARRAY>(
    array: ARRAY,
) -> Aligned<ALIGNMENT, ARRAY>
where
    ALIGNMENT: Alignment,
{
    Aligned {
        _alignment: [],
        array: array,
    }
}

/// `Aligned` constructor
#[allow(non_snake_case)]
#[cfg(not(feature = "const-fn"))]
pub fn Aligned<ALIGNMENT, ARRAY>(
    array: ARRAY,
) -> Aligned<ALIGNMENT, ARRAY>
    where
    ALIGNMENT: Alignment,
{
    Aligned {
        _alignment: [],
        array: array,
    }
}

#[test]
fn sanity() {
    use core::mem;

    let x: Aligned<u16, _> = Aligned([0u8; 3]);
    let y: Aligned<u32, _> = Aligned([0u8; 3]);
    let z: Aligned<u64, _> = Aligned([0u8; 3]);

    assert_eq!(mem::align_of_val(&x), 2);
    assert_eq!(mem::align_of_val(&y), 4);
    assert_eq!(mem::align_of_val(&z), 8);

    assert!(x.as_ptr() as usize % 2 == 0);
    assert!(y.as_ptr() as usize % 4 == 0);
    assert!(z.as_ptr() as usize % 8 == 0);
}
