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

/// Canvas image window coordinates
///
/// Defines a rectangular window on the canvas to export the image from.
///
/// The window origin is defined by its the upper left corner.
///
/// Basic operations
/// ----------------
///
/// ```
/// use planetarium::Window;
///
/// // Create a new rectangular window with origin at (0, 0).
/// let wnd1 = Window::new(128, 64);
///
/// // Move the window origin to (250, 150).
/// let wnd2 = wnd1.at(250, 150);
///
/// // Check the resulting string representation.
/// assert_eq!(wnd2.to_string(), "(250, 150)+(128, 64)");
/// ```
///
/// Conversions
/// -----------
///
/// ```
/// # use planetarium::Window;
/// // From a tuple of tuples representing the origin coordinates
/// // and window dimensions
/// let wnd1 = Window::from(((100, 200), (128, 128)));
///
/// // Check the resulting string representation.
/// assert_eq!(wnd1.to_string(), "(100, 200)+(128, 128)");
/// ```

#[derive(Debug, Clone, Copy)]
pub struct Window {
    /// Window origin X coordinate
    pub x: u32,
    /// Window origin Y coordinate
    pub y: u32,
    /// Width in X direction
    pub w: u32,
    /// Height in Y direction
    pub h: u32,
}

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

impl From<((u32, u32), (u32, u32))> for Window {
    /// Creates a window from a tuple `((x, y), (w, h))`.
    fn from(tuple: ((u32, u32), (u32, u32))) -> Self {
        let ((x, y), (w, h)) = tuple;

        Window { x, y, w, h }
    }
}

impl std::fmt::Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})+({}, {})", self.x, self.y, self.w, self.h)
    }
}

impl std::fmt::Display for EncoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // FIXME: Put full length error descriptions here.
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EncoderError {}

impl Window {
    /// Creates a new window with given dimensions located at the origin.
    pub fn new(width: u32, height: u32) -> Self {
        Window {
            x: 0,
            y: 0,
            w: width,
            h: height,
        }
    }

    /// Moves the window origin to the given origin coordinates.
    pub fn at(&self, x: u32, y: u32) -> Window {
        let w = self.w;
        let h = self.h;

        Window { x, y, w, h }
    }

    /// Checks if the window rectangle is inside the canvas rectangle.
    #[allow(dead_code)]
    fn is_inside(&self, width: u32, height: u32) -> bool {
        self.x + self.w <= width && self.y + self.h <= height
    }

    /// Returns the total number of pixels in the window.
    #[allow(dead_code)]
    fn len(&self) -> usize {
        (self.w * self.h) as usize
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "png"))]
    #[test]
    fn image_format_error() {
        let c = Canvas::new(0, 0);

        assert_eq!(
            c.export_image(ImageFormat::PngGamma8Bpp),
            Err(EncoderError::NotImplemented)
        );
    }

    #[test]
    fn window_ops() {
        let wnd = Window::new(128, 64).at(200, 100);

        assert_eq!(wnd.len(), 128 * 64);
        assert!(wnd.is_inside(400, 500));
        assert!(!wnd.is_inside(100, 100));
        assert!(!wnd.at(300, 100).is_inside(400, 500));
    }
}
