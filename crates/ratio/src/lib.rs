//! The type of ratios and related operations.
//!
//! A **ratio** is the result of dividing two integers, i.e., the numerator and
//! denominator.
//!
//! # Examples
//!
//! ```
//! use ratio::Ratio;
//!
//! let ratio = Ratio::new(1, 3); // 1 / 3
//! assert_eq!(ratio.mul_trunc(20), 6); // trunc(20 * 1 / 3) = trunc(6.66..) = 6
//! assert_eq!(ratio.mul_round(20), 7); // round(20 * 1 / 3) = round(6.66..) = 7
//! println!("{:?}", ratio); // Ratio(1/3 ~= 1431655765/4294967296)
//! ```

#![cfg_attr(not(test), no_std)]

use core::{cmp::PartialEq, fmt};

/// The ratio type.
///
/// It converts `numerator / denominator` to `mult / (1 << shift)` to avoid
/// `u128` division on calculation. The `shift` is  as large as possible to
/// improve precision.
///
/// Currently, it only supports `u32` as the numerator and denominator.
pub struct Ratio {
    numerator: u32,
    denominator: u32,
    mult: u32,
    shift: u32,
}

impl Ratio {
    /// The zero ratio.
    pub const fn zero() -> Self {
        Self {
            numerator: 0,
            denominator: 0,
            mult: 0,
            shift: 0,
        }
    }

    /// Creates a new ratio `numerator / denominator`.
    pub const fn new(numerator: u32, denominator: u32) -> Self {
        assert!(!(denominator == 0 && numerator != 0));
        if numerator == 0 {
            return Self {
                numerator,
                denominator,
                mult: 0,
                shift: 0,
            };
        }

        // numerator / denominator == (numerator * (1 << shift) / denominator) / (1 << shift)
        let mut shift = 32;
        let mut mult;
        loop {
            mult = (((numerator as u64) << shift) + denominator as u64 / 2) / denominator as u64;
            if mult <= u32::MAX as u64 || shift == 0 {
                break;
            }
            shift -= 1;
        }

        while mult % 2 == 0 && shift > 0 {
            mult /= 2;
            shift -= 1;
        }

        Self {
            numerator,
            denominator,
            mult: mult as u32,
            shift,
        }
    }

    /// Get the inverse ratio.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratio::Ratio;
    ///
    /// let ratio = Ratio::new(1, 2);
    /// assert_eq!(ratio.inverse(), Ratio::new(2, 1));
    /// ```
    pub const fn inverse(&self) -> Self {
        Self::new(self.denominator, self.numerator)
    }

    /// Multiplies the ratio by a value and rounds the result down.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratio::Ratio;
    ///
    /// let ratio = Ratio::new(2, 3);
    /// assert_eq!(ratio.mul_trunc(99), 66);  // 99 * 2 / 3 = 66
    /// assert_eq!(ratio.mul_trunc(100), 66); // trunc(100 * 2 / 3) = trunc(66.66...) = 66
    /// ```
    pub const fn mul_trunc(&self, value: u64) -> u64 {
        ((value as u128 * self.mult as u128) >> self.shift) as u64
    }

    /// Multiplies the ratio by a value and rounds the result to the nearest
    /// whole number.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratio::Ratio;
    ///
    /// let ratio = Ratio::new(2, 3);
    /// assert_eq!(ratio.mul_round(99), 66);  // 99 * 2 / 3 = 66
    /// assert_eq!(ratio.mul_round(100), 67); // round(100 * 2 / 3) = round(66.66...) = 67
    /// ```
    pub const fn mul_round(&self, value: u64) -> u64 {
        ((value as u128 * self.mult as u128 + (1 << self.shift >> 1)) >> self.shift) as u64
    }
}

impl fmt::Debug for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Ratio({}/{} ~= {}/{})",
            self.numerator,
            self.denominator,
            self.mult,
            1u64 << self.shift
        )
    }
}

impl PartialEq<Ratio> for Ratio {
    #[inline]
    fn eq(&self, other: &Ratio) -> bool {
        self.mult == other.mult && self.shift == other.shift
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratio() {
        let a = Ratio::new(625_000, 1_000_000);
        let b = Ratio::new(1, u32::MAX);
        let c = Ratio::new(u32::MAX, u32::MAX);
        let d = Ratio::new(u32::MAX, 1);

        assert_eq!(a.mult, 5);
        assert_eq!(a.shift, 3);
        assert_eq!(a.mul_trunc(800), 500);

        assert_eq!(b.mult, 1);
        assert_eq!(b.shift, 32);
        assert_eq!(b.mul_trunc(u32::MAX as _), 0);
        assert_eq!(b.mul_round(u32::MAX as _), 1);

        assert_eq!(c.mult, 1);
        assert_eq!(c.shift, 0);
        assert_eq!(c.mul_trunc(u32::MAX as _), u32::MAX as _);

        println!("{:?}", a);
        println!("{:?}", b);
        println!("{:?}", c);
        println!("{:?}", d);
    }

    #[test]
    fn test_zero() {
        let z1 = Ratio::new(0, 100);
        let z2 = Ratio::zero();
        let z3 = Ratio::new(0, 0);
        assert_eq!(z1.mul_trunc(233), 0);
        assert_eq!(z2.mul_trunc(0), 0);
        assert_eq!(z3.mul_round(456), 0);
    }
}
