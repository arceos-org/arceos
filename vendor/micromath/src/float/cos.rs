//! Cosine approximation. Method from:
//! <https://stackoverflow.com/posts/28050328/revisions>

use super::F32;
use core::f32::consts::FRAC_1_PI;

impl F32 {
    /// Approximates `cos(x)` in radians with a maximum error of `0.002`.
    pub fn cos(self) -> Self {
        let mut x = self;
        x *= FRAC_1_PI / 2.0;
        x -= 0.25 + (x + 0.25).floor().0;
        x *= 16.0 * (x.abs() - 0.5);
        x += 0.225 * x * (x.abs() - 1.0);
        x
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::F32;

    /// Maximum error in radians
    pub(crate) const MAX_ERROR: f32 = 0.002;

    /// Cosine test vectors - `(input, output)`
    const TEST_VECTORS: &[(f32, f32)] = &[
        (0.000, 1.000),
        (0.140, 0.990),
        (0.279, 0.961),
        (0.419, 0.914),
        (0.559, 0.848),
        (0.698, 0.766),
        (0.838, 0.669),
        (0.977, 0.559),
        (1.117, 0.438),
        (1.257, 0.309),
        (1.396, 0.174),
        (1.536, 0.035),
        (1.676, -0.105),
        (1.815, -0.242),
        (1.955, -0.375),
        (2.094, -0.500),
        (2.234, -0.616),
        (2.374, -0.719),
        (2.513, -0.809),
        (2.653, -0.883),
        (2.793, -0.940),
        (2.932, -0.978),
        (3.072, -0.998),
        (3.211, -0.998),
        (3.351, -0.978),
        (3.491, -0.940),
        (3.630, -0.883),
        (3.770, -0.809),
        (3.910, -0.719),
        (4.049, -0.616),
        (4.189, -0.500),
        (4.328, -0.375),
        (4.468, -0.242),
        (4.608, -0.105),
        (4.747, 0.035),
        (4.887, 0.174),
        (5.027, 0.309),
        (5.166, 0.438),
        (5.306, 0.559),
        (5.445, 0.669),
        (5.585, 0.766),
        (5.725, 0.848),
        (5.864, 0.914),
        (6.004, 0.961),
        (6.144, 0.990),
        (6.283, 1.000),
    ];

    #[test]
    fn sanity_check() {
        for &(x, expected) in TEST_VECTORS {
            let cos_x = F32(x).cos();
            let delta = (cos_x - expected).abs();

            assert!(
                delta <= MAX_ERROR,
                "delta {} too large: {} vs {}",
                delta,
                cos_x,
                expected
            );
        }
    }
}
