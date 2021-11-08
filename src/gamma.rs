//! Planetarium
//! ===========
//!
//! Private gamma compression curve definitions
//! -------------------------------------------
//!
//! Defines a new opaque private structure `GammaCurve8`
//! implementing the sRGB gamma compression curve
//! with 8-bit output precision.

/// Opaque 16-bit -> 8-bit gamma compression curve LUT object
pub(crate) struct GammaCurve8 {
    /// LUT byte vector
    lut: Vec<u8>,
}

impl GammaCurve8 {
    /// Lookup table resolution (bits)
    const LUT_BITS: u32 = 12;

    /// Allocates and initializes the gamma compression LUT entries.
    pub(crate) fn new() -> Self {
        let size = 1u32 << Self::LUT_BITS;

        let lut_fn = |i| {
            let x = (i as f32) / ((size - 1) as f32);

            // sRGB gamma curve function
            let gamma = if x <= 0.0031308 {
                // Linear segment
                12.92 * x
            } else {
                // Power-law segment
                1.055 * x.powf(1.0 / 2.4) - 0.055
            };

            (gamma * (u8::MAX as f32) + 0.5) as u8
        };

        let lut = (0..size).map(lut_fn).collect();

        GammaCurve8 { lut }
    }

    /// Converts 16-bit linear light raw samples into
    /// 8-bit gamma-compressed sRGB grayscale samples.
    pub(crate) fn transform(&self, x: u16) -> u8 {
        let shift = 16 - Self::LUT_BITS;
        let i = (x >> shift) as usize;

        debug_assert!(i < self.lut.len());
        unsafe { *self.lut.get_unchecked(i) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform() {
        let lut = GammaCurve8::new();

        assert_eq!(lut.transform(0), 0);
        assert_eq!(lut.transform(16), 1);
        assert_eq!(lut.transform(32), 2);
        assert_eq!(lut.transform(64), 3);
        assert_eq!(lut.transform(80), 4);
        assert_eq!(lut.transform(96), 5);

        assert_eq!(lut.transform(256), 13);
        assert_eq!(lut.transform(1024), 34);
        assert_eq!(lut.transform(16384), 137);
        assert_eq!(lut.transform(32768), 188);
        assert_eq!(lut.transform(65535), 255);
    }
}
