//! Planetarium
//! ===========
//!
//! Private light spot intensity pattern definitions
//! ------------------------------------------------
//!
//! Defines a new opaque private structure `AiryPattern`
//! implementing the intensity function of the Airy disc
//! diffraction pattern as a linear LUT.

// Bessel function of the first kind of order one aka `J1(x)`
use libm::j1f;

/// First positive zero of `J1(x)`
const J1_ZERO1: f32 = 3.831_706;

/// Second positive zero of `J1(x)`
const J1_ZERO2: f32 = 7.015_587;

/// Opaque Airy pattern function LUT object
pub(crate) struct AiryPattern {
    /// LUT samples vector
    lut: Vec<f32>,
}

#[allow(
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
impl AiryPattern {
    /// Fudge factor for the effective spot radius estimation
    ///
    /// The unit radius is the radius of the Airy disc at the first minumum,
    /// also known as the diffraction radius.
    /// The effective (rasterized) spot radius is arbitrarily chosen as
    /// the radius of the second Airy disc minumum.
    pub(crate) const SIZE_FACTOR: f32 = J1_ZERO2 / J1_ZERO1;

    /// Airy intensity pattern LUT size
    const LUT_SIZE: usize = 1024;

    /// Airy intensity pattern LUT size (floating point)
    const LUT_SIZE_FP: f32 = Self::LUT_SIZE as f32;

    /// LUT index to function argument ratio
    const INDEX_SCALE: f32 = Self::LUT_SIZE_FP / Self::SIZE_FACTOR;

    /// Creates the Airy intensity pattern function LUT.
    #[must_use]
    pub(crate) fn new() -> Self {
        let lut_fn = |i| {
            // Resolve singularity at x = 0
            if i > 0 {
                // Airy pattern function argument
                let x = (i as f32) * J1_ZERO2 / Self::LUT_SIZE_FP;

                // Airy disc pattern intensity distribution
                let j1nc = 2.0 * j1f(x) / x;
                j1nc * j1nc
            } else {
                // J1(x) ~ x/2, x -> 0
                1.0
            }
        };

        let lut = (0..Self::LUT_SIZE).map(lut_fn).collect();

        AiryPattern { lut }
    }

    /// Evaluates the Airy intensity pattern function.
    #[must_use]
    pub(crate) fn eval(&self, x: f32) -> f32 {
        // Calculate the LUT index with rounding to the nearest integer.
        let i = (x * Self::INDEX_SCALE + 0.5) as usize;

        // Transparently zero-extend the pattern function LUT to infinity.
        self.lut.get(i).copied().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_lut() {
        let airy = AiryPattern::new();

        // Central maximum
        let f0 = airy.eval(0.0);
        assert!((f0 - 1.0).abs() < 1e-7, "F(0) = {f0}");

        // First zero
        let z1 = 1.0;

        let f1 = airy.eval(z1);
        assert!(f1.abs() < 2e-7, "F({z1}) = {f1}");

        // First zero - eps
        let z1dx = z1 - 2e-3;

        let f1dy = airy.eval(z1dx);
        assert!(f1dy.abs() < 4e-6, "F({z1dx}) = {f1dy}");

        // Second zero
        let z2 = J1_ZERO2 / J1_ZERO1;

        let f2 = airy.eval(z2);
        assert!(f2.abs() < 1e-7, "F({z2}) = {f2}");

        // Second zero - eps
        let z2dx = z2 - 1e-3;

        let f2dy = airy.eval(z2dx);
        assert!(f2dy.abs() < 4e-7, "F({z2dx}) = {f2dy}");

        // Out of range zero < 2LUT
        let z3 = 1.5 * z2;
        let f3 = airy.eval(z3);
        assert!(f3.abs() < 1e-7);

        // Out of range zero > 2LUT
        let z4 = 3.5 * z2;
        let f4 = airy.eval(z4);
        assert!(f4.abs() < 1e-7);

        // assert!(false, "T = {:?}", airy.lut)
    }
}
