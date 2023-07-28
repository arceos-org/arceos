// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use core::fmt::{Debug, Error, Formatter};
use core::hash::{Hash, Hasher};
use core::mem::{size_of, MaybeUninit};
use core::ops::*;

use crate::types::{BitOps, Bits, BitsImpl};

/// A compact array of bits.
///
/// The type used to store the bitmap will be the minimum unsigned integer type
/// required to fit the number of bits, from `u8` to `u128`. If the size is 1,
/// `bool` is used. If the size exceeds 128, an array of `u128` will be used,
/// sized as appropriately. The maximum supported size is currently 1024,
/// represented by an array `[u128; 8]`.
pub struct Bitmap<const SIZE: usize>
where
    BitsImpl<{ SIZE }>: Bits,
{
    pub(crate) data: <BitsImpl<{ SIZE }> as Bits>::Store,
}

impl<const SIZE: usize> Clone for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    fn clone(&self) -> Self {
        Bitmap::from_value(self.data)
    }
}

impl<const SIZE: usize> Copy for Bitmap<{ SIZE }> where BitsImpl<{ SIZE }>: Bits {}

impl<const SIZE: usize> Default for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    fn default() -> Self {
        Bitmap {
            data: <BitsImpl<SIZE> as Bits>::Store::default(),
        }
    }
}

impl<const SIZE: usize> PartialEq for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<const SIZE: usize> Eq for Bitmap<{ SIZE }> where BitsImpl<{ SIZE }>: Bits {}

impl<const SIZE: usize> Hash for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
    <BitsImpl<{ SIZE }> as Bits>::Store: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_value().hash(state)
    }
}

impl<const SIZE: usize> PartialOrd for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
    <BitsImpl<{ SIZE }> as Bits>::Store: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.as_value().partial_cmp(other.as_value())
    }
}

impl<const SIZE: usize> Ord for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
    <BitsImpl<{ SIZE }> as Bits>::Store: Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_value().cmp(other.as_value())
    }
}

#[cfg(feature = "std")]
impl<const SIZE: usize> Debug for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            <BitsImpl::<SIZE> as Bits>::Store::to_hex(&self.data)
        )
    }
}

impl<const SIZE: usize> AsRef<[u8]> for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    fn as_ref(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                &self.data as *const _ as *const u8,
                size_of::<<BitsImpl<SIZE> as Bits>::Store>(),
            )
        }
    }
}

impl<const SIZE: usize> AsMut<[u8]> for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                &mut self.data as *mut _ as *mut u8,
                size_of::<<BitsImpl<SIZE> as Bits>::Store>(),
            )
        }
    }
}

impl<const SIZE: usize> TryFrom<&[u8]> for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == size_of::<<BitsImpl<SIZE> as Bits>::Store>() {
            let mut data: MaybeUninit<<BitsImpl<SIZE> as Bits>::Store> = MaybeUninit::uninit();
            let data_ptr: *mut u8 = data.as_mut_ptr().cast();
            Ok(unsafe {
                data_ptr.copy_from_nonoverlapping(value.as_ptr(), value.len());
                Self {
                    data: data.assume_init(),
                }
            })
        } else {
            Err(())
        }
    }
}

#[cfg(not(feature = "std"))]
impl<const SIZE: usize> Debug for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Bitmap<{}> {{ ... }}", SIZE)
    }
}

impl<const SIZE: usize> Bitmap<{ SIZE }>
where
    BitsImpl<SIZE>: Bits,
{
    /// Construct a bitmap with every bit set to `false`.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a bitmap where every bit with index less than `bits` is
    /// `true`, and every other bit is `false`.
    #[inline]
    pub fn mask(bits: usize) -> Self {
        debug_assert!(bits <= SIZE);
        Self {
            data: <BitsImpl<SIZE> as Bits>::Store::make_mask(bits),
        }
    }

    /// Construct a bitmap from a value of the same type as its backing store.
    #[inline]
    pub fn from_value(data: <BitsImpl<SIZE> as Bits>::Store) -> Self {
        Self { data }
    }

    /// Convert this bitmap into a value of the type of its backing store.
    #[inline]
    pub fn into_value(self) -> <BitsImpl<SIZE> as Bits>::Store {
        self.data
    }

    /// Get a reference to this bitmap's backing store.
    #[inline]
    pub fn as_value(&self) -> &<BitsImpl<SIZE> as Bits>::Store {
        &self.data
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        AsRef::<[u8]>::as_ref(self)
    }

    /// Count the number of `true` bits in the bitmap.
    #[inline]
    pub fn len(self) -> usize {
        <BitsImpl<SIZE> as Bits>::Store::len(&self.data)
    }

    /// Test if the bitmap contains only `false` bits.
    #[inline]
    pub fn is_empty(self) -> bool {
        self.first_index().is_none()
    }

    /// Test if the bitmap contains only `true` bits.
    #[inline]
    pub fn is_full(self) -> bool {
        self.first_false_index().is_none()
    }

    /// Get the value of the bit at a given index.
    #[inline]
    pub fn get(self, index: usize) -> bool {
        debug_assert!(index < SIZE);
        <BitsImpl<SIZE> as Bits>::Store::get(&self.data, index)
    }

    /// Set the value of the bit at a given index.
    ///
    /// Returns the previous value of the bit.
    #[inline]
    pub fn set(&mut self, index: usize, value: bool) -> bool {
        debug_assert!(index < SIZE);
        <BitsImpl<SIZE> as Bits>::Store::set(&mut self.data, index, value)
    }

    /// Find the index of the first `true` bit in the bitmap.
    #[inline]
    pub fn first_index(self) -> Option<usize> {
        <BitsImpl<SIZE> as Bits>::Store::first_index(&self.data)
    }

    /// Find the index of the last `true` bit in the bitmap.
    #[inline]
    pub fn last_index(self) -> Option<usize> {
        <BitsImpl<SIZE> as Bits>::Store::last_index(&self.data)
    }

    /// Find the index of the first `true` bit in the bitmap after `index`.
    #[inline]
    pub fn next_index(self, index: usize) -> Option<usize> {
        <BitsImpl<SIZE> as Bits>::Store::next_index(&self.data, index)
    }

    /// Find the index of the last `true` bit in the bitmap before `index`.
    #[inline]
    pub fn prev_index(self, index: usize) -> Option<usize> {
        <BitsImpl<SIZE> as Bits>::Store::prev_index(&self.data, index)
    }

    /// Find the index of the first `false` bit in the bitmap.
    #[inline]
    pub fn first_false_index(self) -> Option<usize> {
        <BitsImpl<SIZE> as Bits>::corrected_first_false_index(&self.data)
    }

    /// Find the index of the last `false` bit in the bitmap.
    #[inline]
    pub fn last_false_index(self) -> Option<usize> {
        <BitsImpl<SIZE> as Bits>::corrected_last_false_index(&self.data)
    }

    /// Find the index of the first `false` bit in the bitmap after `index`.
    #[inline]
    pub fn next_false_index(self, index: usize) -> Option<usize> {
        <BitsImpl<SIZE> as Bits>::corrected_next_false_index(&self.data, index)
    }

    /// Find the index of the first `false` bit in the bitmap before `index`.
    #[inline]
    pub fn prev_false_index(self, index: usize) -> Option<usize> {
        <BitsImpl<SIZE> as Bits>::Store::prev_false_index(&self.data, index)
    }

    /// Invert all the bits in the bitmap.
    #[inline]
    pub fn invert(&mut self) {
        <BitsImpl<SIZE> as Bits>::Store::invert(&mut self.data);
    }
}

impl<'a, const SIZE: usize> IntoIterator for &'a Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    type Item = usize;
    type IntoIter = Iter<'a, { SIZE }>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            head: None,
            tail: Some(SIZE + 1),
            data: self,
        }
    }
}

impl<const SIZE: usize> BitAnd for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        <BitsImpl<SIZE> as Bits>::Store::bit_and(&mut self.data, &rhs.data);
        self
    }
}

impl<const SIZE: usize> BitOr for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        <BitsImpl<SIZE> as Bits>::Store::bit_or(&mut self.data, &rhs.data);
        self
    }
}

impl<const SIZE: usize> BitXor for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        <BitsImpl<SIZE> as Bits>::Store::bit_xor(&mut self.data, &rhs.data);
        self
    }
}

impl<const SIZE: usize> Not for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    type Output = Self;
    fn not(mut self) -> Self::Output {
        <BitsImpl<SIZE> as Bits>::Store::invert(&mut self.data);
        self
    }
}

impl<const SIZE: usize> BitAndAssign for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    fn bitand_assign(&mut self, rhs: Self) {
        <BitsImpl<SIZE> as Bits>::Store::bit_and(&mut self.data, &rhs.data);
    }
}

impl<const SIZE: usize> BitOrAssign for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    fn bitor_assign(&mut self, rhs: Self) {
        <BitsImpl<SIZE> as Bits>::Store::bit_or(&mut self.data, &rhs.data);
    }
}

impl<const SIZE: usize> BitXorAssign for Bitmap<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    fn bitxor_assign(&mut self, rhs: Self) {
        <BitsImpl<SIZE> as Bits>::Store::bit_xor(&mut self.data, &rhs.data);
    }
}

impl From<[u128; 2]> for Bitmap<256> {
    fn from(data: [u128; 2]) -> Self {
        Bitmap { data }
    }
}

impl From<[u128; 3]> for Bitmap<384> {
    fn from(data: [u128; 3]) -> Self {
        Bitmap { data }
    }
}

impl From<[u128; 4]> for Bitmap<512> {
    fn from(data: [u128; 4]) -> Self {
        Bitmap { data }
    }
}

impl From<[u128; 5]> for Bitmap<640> {
    fn from(data: [u128; 5]) -> Self {
        Bitmap { data }
    }
}

impl From<[u128; 6]> for Bitmap<768> {
    fn from(data: [u128; 6]) -> Self {
        Bitmap { data }
    }
}

impl From<[u128; 7]> for Bitmap<896> {
    fn from(data: [u128; 7]) -> Self {
        Bitmap { data }
    }
}

impl From<[u128; 8]> for Bitmap<1024> {
    fn from(data: [u128; 8]) -> Self {
        Bitmap { data }
    }
}

impl From<Bitmap<256>> for [u128; 2] {
    fn from(bitmap: Bitmap<256>) -> Self {
        bitmap.into_value()
    }
}

impl From<Bitmap<384>> for [u128; 3] {
    fn from(bitmap: Bitmap<384>) -> Self {
        bitmap.into_value()
    }
}

impl From<Bitmap<512>> for [u128; 4] {
    fn from(bitmap: Bitmap<512>) -> Self {
        bitmap.into_value()
    }
}

impl From<Bitmap<640>> for [u128; 5] {
    fn from(bitmap: Bitmap<640>) -> Self {
        bitmap.into_value()
    }
}

impl From<Bitmap<768>> for [u128; 6] {
    fn from(bitmap: Bitmap<768>) -> Self {
        bitmap.into_value()
    }
}

impl From<Bitmap<896>> for [u128; 7] {
    fn from(bitmap: Bitmap<896>) -> Self {
        bitmap.into_value()
    }
}

impl From<Bitmap<1024>> for [u128; 8] {
    fn from(bitmap: Bitmap<1024>) -> Self {
        bitmap.into_value()
    }
}

/// An iterator over the indices in a bitmap which are `true`.
///
/// This yields a sequence of `usize` indices, not their contents (which are
/// always `true` anyway, by definition).
///
/// # Examples
///
/// ```rust
/// # use bitmaps::Bitmap;
/// let mut bitmap: Bitmap<10> = Bitmap::new();
/// bitmap.set(3, true);
/// bitmap.set(5, true);
/// bitmap.set(8, true);
/// let true_indices: Vec<usize> = bitmap.into_iter().collect();
/// assert_eq!(vec![3, 5, 8], true_indices);
/// ```
#[derive(Clone, Debug)]
pub struct Iter<'a, const SIZE: usize>
where
    BitsImpl<SIZE>: Bits,
{
    head: Option<usize>,
    tail: Option<usize>,
    data: &'a Bitmap<{ SIZE }>,
}

impl<'a, const SIZE: usize> Iterator for Iter<'a, SIZE>
where
    BitsImpl<{ SIZE }>: Bits,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let result;

        match self.head {
            None => {
                result = self.data.first_index();
            }
            Some(index) => {
                if index >= SIZE {
                    result = None
                } else {
                    result = self.data.next_index(index);
                }
            }
        }

        if let Some(index) = result {
            if let Some(tail) = self.tail {
                if tail < index {
                    self.head = Some(SIZE + 1);
                    self.tail = None;
                    return None;
                }
            } else {
                // tail is already done
                self.head = Some(SIZE + 1);
                return None;
            }

            self.head = Some(index);
        } else {
            self.head = Some(SIZE + 1);
        }

        result
    }
}

impl<'a, const SIZE: usize> DoubleEndedIterator for Iter<'a, SIZE>
where
    BitsImpl<{ SIZE }>: Bits,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let result;

        match self.tail {
            None => {
                result = None;
            }
            Some(index) => {
                if index >= SIZE {
                    result = self.data.last_index();
                } else {
                    result = self.data.prev_index(index);
                }
            }
        }

        if let Some(index) = result {
            if let Some(head) = self.head {
                if head > index {
                    self.head = Some(SIZE + 1);
                    self.tail = None;
                    return None;
                }
            }

            self.tail = Some(index);
        } else {
            self.tail = None;
        }

        result
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[allow(clippy::cast_ptr_alignment)]
mod x86_arch {
    use super::*;
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    impl Bitmap<128> {
        #[target_feature(enable = "sse2")]
        pub unsafe fn load_m128i(&self) -> __m128i {
            _mm_loadu_si128(&self.data as *const _ as *const __m128i)
        }
    }

    impl Bitmap<256> {
        #[target_feature(enable = "sse2")]
        pub unsafe fn load_m128i(&self) -> [__m128i; 2] {
            let ptr = &self.data as *const _ as *const __m128i;
            [_mm_loadu_si128(ptr), _mm_loadu_si128(ptr.add(1))]
        }

        #[target_feature(enable = "avx")]
        pub unsafe fn load_m256i(&self) -> __m256i {
            _mm256_loadu_si256(&self.data as *const _ as *const __m256i)
        }
    }

    impl Bitmap<512> {
        #[target_feature(enable = "sse2")]
        pub unsafe fn load_m128i(&self) -> [__m128i; 4] {
            let ptr = &self.data as *const _ as *const __m128i;
            [
                _mm_loadu_si128(ptr),
                _mm_loadu_si128(ptr.add(1)),
                _mm_loadu_si128(ptr.add(2)),
                _mm_loadu_si128(ptr.add(3)),
            ]
        }

        #[target_feature(enable = "avx")]
        pub unsafe fn load_m256i(&self) -> [__m256i; 2] {
            let ptr = &self.data as *const _ as *const __m256i;
            [_mm256_loadu_si256(ptr), _mm256_loadu_si256(ptr.add(1))]
        }
    }

    impl Bitmap<768> {
        #[target_feature(enable = "sse2")]
        pub unsafe fn load_m128i(&self) -> [__m128i; 6] {
            let ptr = &self.data as *const _ as *const __m128i;
            [
                _mm_loadu_si128(ptr),
                _mm_loadu_si128(ptr.add(1)),
                _mm_loadu_si128(ptr.add(2)),
                _mm_loadu_si128(ptr.add(3)),
                _mm_loadu_si128(ptr.add(4)),
                _mm_loadu_si128(ptr.add(5)),
            ]
        }

        #[target_feature(enable = "avx")]
        pub unsafe fn load_m256i(&self) -> [__m256i; 3] {
            let ptr = &self.data as *const _ as *const __m256i;
            [
                _mm256_loadu_si256(ptr),
                _mm256_loadu_si256(ptr.add(1)),
                _mm256_loadu_si256(ptr.add(2)),
            ]
        }
    }

    impl Bitmap<1024> {
        #[target_feature(enable = "sse2")]
        pub unsafe fn load_m128i(&self) -> [__m128i; 8] {
            let ptr = &self.data as *const _ as *const __m128i;
            [
                _mm_loadu_si128(ptr),
                _mm_loadu_si128(ptr.add(1)),
                _mm_loadu_si128(ptr.add(2)),
                _mm_loadu_si128(ptr.add(3)),
                _mm_loadu_si128(ptr.add(4)),
                _mm_loadu_si128(ptr.add(5)),
                _mm_loadu_si128(ptr.add(6)),
                _mm_loadu_si128(ptr.add(7)),
            ]
        }

        #[target_feature(enable = "avx")]
        pub unsafe fn load_m256i(&self) -> [__m256i; 4] {
            let ptr = &self.data as *const _ as *const __m256i;
            [
                _mm256_loadu_si256(ptr),
                _mm256_loadu_si256(ptr.add(1)),
                _mm256_loadu_si256(ptr.add(2)),
                _mm256_loadu_si256(ptr.add(3)),
            ]
        }
    }

    impl From<__m128i> for Bitmap<128> {
        fn from(data: __m128i) -> Self {
            Self {
                data: unsafe { core::mem::transmute(data) },
            }
        }
    }

    impl From<__m256i> for Bitmap<256> {
        fn from(data: __m256i) -> Self {
            Self {
                data: unsafe { core::mem::transmute(data) },
            }
        }
    }

    impl From<Bitmap<128>> for __m128i {
        fn from(data: Bitmap<128>) -> Self {
            unsafe { data.load_m128i() }
        }
    }

    impl From<Bitmap<256>> for __m256i {
        fn from(data: Bitmap<256>) -> Self {
            unsafe { data.load_m256i() }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        struct AlignmentTester<const B: usize>
        where
            BitsImpl<B>: Bits,
        {
            _byte: u8,
            bits: Bitmap<B>,
        }

        #[test]
        fn load_128() {
            let mut t: AlignmentTester<128> = AlignmentTester {
                _byte: 0,
                bits: Bitmap::new(),
            };
            t.bits.set(5, true);
            let m = unsafe { t.bits.load_m128i() };
            let mut bits: Bitmap<128> = m.into();
            assert!(bits.set(5, false));
            assert!(bits.is_empty());
        }

        #[test]
        fn load_256() {
            let mut t: AlignmentTester<256> = AlignmentTester {
                _byte: 0,
                bits: Bitmap::new(),
            };
            t.bits.set(5, true);
            let m = unsafe { t.bits.load_m256i() };
            let mut bits: Bitmap<256> = m.into();
            assert!(bits.set(5, false));
            assert!(bits.is_empty());
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::collection::btree_set;
    use proptest::proptest;

    proptest! {
        #[test]
        fn get_set_and_iter_61(bits in btree_set(0..61usize, 0..61)) {
            let mut bitmap = Bitmap::<61>::new();
            for i in &bits {
                bitmap.set(*i, true);
            }
            for i in 0..61 {
                assert_eq!(bitmap.get(i), bits.contains(&i));
            }
            assert_eq!(bitmap.first_index(), bits.clone().into_iter().next());
            assert_eq!(bitmap.last_index(), bits.clone().into_iter().next_back());
            assert!(bitmap.into_iter().eq(bits.clone().into_iter()));
            assert!(bitmap.into_iter().rev().eq(bits.into_iter().rev()));
        }

        #[test]
        fn get_set_and_iter_64(bits in btree_set(0..64usize, 0..64)) {
            let mut bitmap = Bitmap::<64>::new();
            for i in &bits {
                bitmap.set(*i, true);
            }
            for i in 0..64 {
                assert_eq!(bitmap.get(i), bits.contains(&i));
            }
            assert_eq!(bitmap.first_index(), bits.clone().into_iter().next());
            assert_eq!(bitmap.last_index(), bits.clone().into_iter().next_back());
            assert!(bitmap.into_iter().eq(bits.clone().into_iter()));
            assert!(bitmap.into_iter().rev().eq(bits.into_iter().rev()));
        }

        #[test]
        fn get_set_and_iter_1024(bits in btree_set(0..1024usize, 0..1024)) {
            let mut bitmap = Bitmap::<1024>::new();
            for i in &bits {
                bitmap.set(*i, true);
            }
            for i in 0..1024 {
                assert_eq!(bitmap.get(i), bits.contains(&i));
            }
            assert_eq!(bitmap.first_index(), bits.clone().into_iter().next());
            assert_eq!(bitmap.last_index(), bits.clone().into_iter().next_back());
            assert!(bitmap.into_iter().eq(bits.clone().into_iter()));
            assert!(bitmap.into_iter().rev().eq(bits.into_iter().rev()));
        }

        #[test]
        fn convert_1024(bits in btree_set(0..1024usize, 0..1024)) {
            let mut bitmap = Bitmap::<1024>::new();
            for i in &bits {
                bitmap.set(*i, true);
            }
            let new_bitmap: Bitmap<1024> = TryFrom::try_from(bitmap.as_bytes()).expect("Unable to convert bitmap!");
            assert_eq!(new_bitmap, bitmap);
        }
    }
}
