//! Planetarium
//! ===========
//!
//! Canvas image export support definitions
//! ---------------------------------------
//!
//! Defines an enum for the supported image export formats
//! and image export methods for `Canvas`.
//!
//! Defines a custom error enum type `EncoderError`.

mod raw;

#[cfg(feature = "png")]
mod png;

use crate::Canvas;

/// Exportable canvas image formats
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ImageFormat {
    // Internal encoders:
    /// 8-bit gamma-compressed grayscale RAW
    RawGamma8Bpp,
    /// 10-bit linear light grayscale little-endian RAW
    RawLinear10BppLE,
    /// 12-bit linear light grayscale little-endian RAW
    RawLinear12BppLE,

    // Require "png" feature:
    /// 8-bit gamma-compressed grayscale PNG
    PngGamma8Bpp,
    /// 16-bit linear light grayscale PNG
    PngLinear16Bpp,
}

/// Image export encoder error type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EncoderError {
    /// Requested image format not supported
    NotImplemented,
}

impl std::fmt::Display for EncoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // FIXME: Put full length error descriptions here.
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EncoderError {}

impl Canvas {
    /// Exports the canvas contents in the requested image format.
    #[cfg(not(feature = "png"))]
    pub fn export_image(&self, format: ImageFormat) -> Result<Vec<u8>, EncoderError> {
        match format {
            ImageFormat::RawGamma8Bpp => self.export_raw8bpp(),
            ImageFormat::RawLinear10BppLE => self.export_raw1xbpp::<10>(),
            ImageFormat::RawLinear12BppLE => self.export_raw1xbpp::<12>(),
            _ => Err(EncoderError::NotImplemented),
        }
    }

    /// Exports the canvas contents in the requested image format.
    #[cfg(feature = "png")]
    pub fn export_image(&self, format: ImageFormat) -> Result<Vec<u8>, EncoderError> {
        match format {
            ImageFormat::RawGamma8Bpp => self.export_raw8bpp(),
            ImageFormat::RawLinear10BppLE => self.export_raw1xbpp::<10>(),
            ImageFormat::RawLinear12BppLE => self.export_raw1xbpp::<12>(),
            ImageFormat::PngGamma8Bpp => self.export_png8bpp(),
            ImageFormat::PngLinear16Bpp => self.export_png16bpp(),
        }
    }
}

#[cfg(all(test, not(feature = "png")))]
mod tests {
    use super::*;

    #[test]
    fn image_format_error() {
        let c = Canvas::new(0, 0);

        assert_eq!(
            c.export_image(ImageFormat::PngGamma8Bpp),
            Err(EncoderError::NotImplemented)
        );
    }
}
