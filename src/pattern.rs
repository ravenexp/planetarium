//! Planetarium
//! ===========
//!
//! Private light spot intensity pattern definitions
//! ------------------------------------------------
//!
//! Contains private types and implementations of private methods
//! for the existing public types.

// Bessel function of the first kind of order one aka `J1(x)`
use libm::j1f;

use super::Canvas;

/// First positive zero of `J1(x)`
pub(crate) const J1_ZERO1: f32 = 3.831706;

/// Second positive zero of `J1(x)`
pub(crate) const J1_ZERO2: f32 = 7.015587;

impl Canvas {
    /// Light spot intensity pattern LUT size
    const PATTERN_LUT_SIZE: usize = 256;

    /// Builds the light spot intensity pattern LUT.
    ///
    /// Returns the LUT array and the table index scaling factor.
    pub(super) fn build_pattern_lut() -> (Vec<f32>, f32) {
        let max_index = (Self::PATTERN_LUT_SIZE - 1) as f32;
        let pattern_scale = max_index / (J1_ZERO2 / J1_ZERO1);
        let indexes = 0..Self::PATTERN_LUT_SIZE;

        let lut_fn = |i| {
            // Resolve singularity at x = 0
            if i > 0 {
                // Airy pattern function argument
                let x = (i as f32) * J1_ZERO2 / (Self::PATTERN_LUT_SIZE as f32);

                // Airy disc pattern intensity distribution
                let j1nc = 2.0 * j1f(x) / x;
                j1nc * j1nc
            } else {
                // J1(x) ~ x/2, x -> 0
                1.0
            }
        };

        let pattern_lut = indexes.map(lut_fn).collect();

        (pattern_lut, pattern_scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_lut() {
        let (lut, k) = Canvas::build_pattern_lut();
        let lut_fn = |x: f32| lut[(k * x) as usize];

        // Central maximum
        let f0 = lut_fn(0.0);
        assert!((f0 - 1.0).abs() < 1e-6, "F(0) = {}", f0);

        // First zero
        let f1 = lut_fn(1.0);
        assert!(f1.abs() < 1e-4, "F(1) = {}", f1);

        // Second zero
        let z2 = J1_ZERO2 / J1_ZERO1;
        let f2 = lut_fn(z2);
        assert!(f2.abs() < 1e-4, "F({}) = {}", z2, f2);
    }
}
