//! Adapted from the `madgwick` crate: <https://github.com/japaric/madgwick>
//! Copyright (c) 2018 Jorge Aparicio
//!
//! Original sources dual licensed under your choice of the Apache 2.0
//! and/or MIT licenses, which matches this crate's licensing terms.
//!
//! See toplevel LICENSE-MIT for more information on the MIT license.
//! Apache 2.0 license follows:
//!
//! Licensed under the Apache License, Version 2.0 (the "License");
//! you may not use this file except in compliance with the License.
//! You may obtain a copy of the License at:
//!
//! <https://www.apache.org/licenses/LICENSE-2.0>
//!
//! Unless required by applicable law or agreed to in writing, software
//! distributed under the License is distributed on an "AS IS" BASIS,
//! WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//! See the License for the specific language governing permissions and
//! limitations under the License.

use crate::F32;
use core::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[cfg(feature = "vector")]
use crate::vector::{Component, F32x3, Vector3d};

/// Quaternions are a number system that extends the complex numbers which can
/// be used for efficiently computing spatial rotations.
///
/// They're computed as the quotient of two directed lines in a
/// three-dimensional space, or equivalently as the quotient of two vectors.
///
/// For given real numbers `a`, `b`, `c`, and `d`, they take the form:
///
/// `a + bi + cj + dk`
///
/// where `i`, `j`, and `k` are the fundamental quaternion units:
///
/// `i² = j² = k² = i*j*k = -1`
///
/// Quaternion multiplication is non-commutative:
///
/// | x | 1  | i  | j  | k  |
/// |---|----|----|----|----|
/// | 1 | 1  | i  | j  | k  |
/// | i | i  | -1 | k  | -j |
/// | j | j  | -k | -1 | i  |
/// | k | k  | j  | -i | -1 |
#[cfg_attr(docsrs, doc(cfg(feature = "quaternion")))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion(f32, f32, f32, f32);

impl Quaternion {
    /// Identity quaternion.
    pub const IDENTITY: Self = Self(1.0, 0.0, 0.0, 0.0);

    /// Create a new quaternion.
    pub const fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self(a, b, c, d)
    }

    /// Returns the conjugate of this quaternion.
    pub fn conj(self) -> Self {
        Quaternion(self.0, -self.1, -self.2, -self.3)
    }

    /// Returns the dot product of this quaternion.
    pub fn dot(self, rhs: Self) -> f32 {
        self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2 + self.3 * rhs.3
    }

    /// Compute the inverse of this quaternion.
    ///
    /// Panics if [`Quaternion::norm`] is zero.
    pub fn inv(self) -> Self {
        let norm = self.norm();
        assert_ne!(norm, 0.0, "quaternion norm is zero");
        self.conj() * F32(norm).inv().0
    }

    /// Compute the magnitude (a.k.a length) of this quaternion.
    pub fn magnitude(self) -> f32 {
        F32(self.norm()).sqrt().0
    }

    /// Returns the norm of this quaternion, i.e. `a²+b²+c²+d²`.
    ///
    /// <https://www.mathworks.com/help/aeroblks/quaternionnorm.html>
    pub fn norm(self) -> f32 {
        self.0 * self.0 + self.1 * self.1 + self.2 * self.2 + self.3 * self.3
    }

    /// Compute a quaternion for the given axis vector and angle.
    #[cfg_attr(docsrs, doc(cfg(feature = "vector")))]
    pub fn axis_angle<C>(v: Vector3d<C>, theta: C) -> Self
    where
        C: Component + Into<f32>,
    {
        let half_theta = F32(theta.into() * 0.5);

        // TODO(tarcieri): refactor `Quaternion` to be (f32 + F32x3)
        let v = F32x3 {
            x: v.x.into(),
            y: v.y.into(),
            z: v.z.into(),
        } * half_theta.sin().0;

        Self(half_theta.cos().0, v.x, v.y, v.z)
    }

    /// Rotate a 3D vector using this quaternion.
    #[cfg_attr(docsrs, doc(cfg(feature = "vector")))]
    pub fn rotate<C>(self, v: Vector3d<C>) -> F32x3
    where
        C: Component + Into<f32>,
    {
        let q = self * Quaternion::from(v) * self.inv();

        F32x3 {
            x: q.1,
            y: q.2,
            z: q.3,
        }
    }

    /// Scale by a scalar.
    pub fn scale<S>(self, scalar: S) -> Self
    where
        S: Into<f32>,
    {
        let k = scalar.into();
        Self(self.0 * k, self.1 * k, self.2 * k, self.3 * k)
    }

    /// Convert this quaternion into an array.
    pub fn to_array(&self) -> [f32; 4] {
        [self.0, self.1, self.2, self.3]
    }
}

impl Add for Quaternion {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(
            self.0 + rhs.0,
            self.1 + rhs.1,
            self.2 + rhs.2,
            self.3 + rhs.3,
        )
    }
}

impl AddAssign for Quaternion {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Default for Quaternion {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl From<(f32, f32, f32, f32)> for Quaternion {
    fn from(q: (f32, f32, f32, f32)) -> Quaternion {
        Self::new(q.0, q.1, q.2, q.3)
    }
}

impl From<Quaternion> for (f32, f32, f32, f32) {
    fn from(q: Quaternion) -> (f32, f32, f32, f32) {
        (q.0, q.1, q.2, q.3)
    }
}

impl Mul for Quaternion {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(
            self.0 * other.0 - self.1 * other.1 - self.2 * other.2 - self.3 * other.3,
            self.0 * other.1 + self.1 * other.0 + self.2 * other.3 - self.3 * other.2,
            self.0 * other.2 - self.1 * other.3 + self.2 * other.0 + self.3 * other.1,
            self.0 * other.3 + self.1 * other.2 - self.2 * other.1 + self.3 * other.0,
        )
    }
}

impl Mul<f32> for Quaternion {
    type Output = Self;

    fn mul(self, k: f32) -> Self {
        self.scale(k)
    }
}

impl Mul<Quaternion> for f32 {
    type Output = Quaternion;

    fn mul(self, q: Quaternion) -> Quaternion {
        q.scale(self)
    }
}

impl MulAssign<f32> for Quaternion {
    fn mul_assign(&mut self, k: f32) {
        *self = *self * k;
    }
}

impl Sub for Quaternion {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(
            self.0 - rhs.0,
            self.1 - rhs.1,
            self.2 - rhs.2,
            self.3 - rhs.3,
        )
    }
}

impl SubAssign for Quaternion {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

#[cfg(feature = "vector")]
#[cfg_attr(docsrs, doc(cfg(feature = "vector")))]
impl<C> From<Vector3d<C>> for Quaternion
where
    C: Component + Into<f32>,
{
    fn from(v: Vector3d<C>) -> Quaternion {
        Self(0.0, v.x.into(), v.y.into(), v.z.into())
    }
}

#[cfg(feature = "vector")]
#[cfg_attr(docsrs, doc(cfg(feature = "vector")))]
impl<C> Mul<Vector3d<C>> for Quaternion
where
    C: Component + Into<f32>,
{
    type Output = F32x3;

    fn mul(self, v: Vector3d<C>) -> F32x3 {
        self.rotate(v)
    }
}

#[cfg(test)]
mod tests {
    use super::Quaternion;
    use crate::F32Ext;

    const MAX_ERROR: f32 = 0.05;

    #[test]
    fn conj_test() {
        let q = Quaternion(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q.conj(), Quaternion(1.0, -2.0, -3.0, -4.0));
    }

    #[test]
    fn norm_test() {
        let q = Quaternion(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q.norm(), 30.0);

        let n = q.norm().invsqrt();
        let r = q * n;

        // The magnitude of the norm should be 1.0
        let allowed_delta = 1.0 * MAX_ERROR;
        let actual_delta = (r.norm() - 1.0).abs();

        assert!(
            actual_delta <= allowed_delta,
            "delta {} too large: {} vs {}",
            actual_delta,
            r.norm(),
            1.0
        );
    }

    #[test]
    fn add_assign() {
        let mut q = Quaternion(1.0, 2.0, 3.0, 4.0);
        q += Quaternion(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q, Quaternion(2.0, 4.0, 6.0, 8.0));
    }

    #[test]
    fn mul_assign() {
        let mut q = Quaternion(1.0, 2.0, 3.0, 4.0);
        q *= 2.0;
        assert_eq!(q, Quaternion(2.0, 4.0, 6.0, 8.0));
    }

    #[test]
    fn sub_assign() {
        let mut q = Quaternion(2.0, 4.0, 6.0, 8.0);
        q -= Quaternion(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q, Quaternion(1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn mul_quaternion() {
        let q = Quaternion(1.0, 2.0, 3.0, 4.0);
        let r = Quaternion(4.0, 3.0, 2.0, 1.0);
        assert_eq!(q * r, Quaternion(-12.0, 6.0, 24.0, 12.0));
    }

    #[test]
    fn mul_f32() {
        let q = Quaternion(1.0, 2.0, 3.0, 4.0);
        let r = 2.0 * q;
        assert_eq!(r, Quaternion(2.0, 4.0, 6.0, 8.0));

        let s = r * 0.5;
        assert_eq!(s, Quaternion(1.0, 2.0, 3.0, 4.0));
    }
}
