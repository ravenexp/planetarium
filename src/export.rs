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

use crate::{Canvas, Pixel};

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
    /// Requested image window is out of bounds
    BrokenWindow,
}

/// Canvas window image scanlines iterator
///
/// Yields the window image pixel spans as `&[Pixel]` slices.
///
/// Usage
/// -----
///
/// ```
/// use planetarium::{Canvas, Window};
///
/// let c = Canvas::new(100, 100);
///
/// // Define a 10x10 window rectangle with origin at (50, 50).
/// let wnd = Window::new(10, 10).at(50, 50);
///
/// // Iterate over the window image scanlines yielding 10-pixel spans.
/// for span in c.window_spans(wnd).unwrap() {
///     // Dummy check
///     assert_eq!(span, [0u16; 10]);
/// }
/// ```
pub struct WindowSpans<'a> {
    /// Source canvas object
    canvas: &'a Canvas,

    /// Canvas window rectangle
    window: Window,

    /// Current scanline index
    scanline: u32,
}

impl<'a> Iterator for WindowSpans<'a> {
    /// Image pixel span type
    type Item = &'a [Pixel];

    /// Iterates over the window image scanlines and returns the resulting
    /// image pixel spans as `&'a [Pixel]`.
    fn next(&mut self) -> Option<Self::Item> {
        // Terminate when the current scanline is outside of the window rectangle.
        if self.scanline >= self.window.y + self.window.h {
            return None;
        }

        // Calculate the current pixel span indexes.
        let base = (self.canvas.width * self.scanline + self.window.x) as usize;
        let end = base + self.window.w as usize;

        self.scanline += 1;

        Some(&self.canvas.pixbuf[base..end])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = (self.window.y + self.window.h - self.scanline) as usize;

        (size, Some(size))
    }
}

impl<'a> ExactSizeIterator for WindowSpans<'a> {}

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
        write!(f, "{self:?}")
    }
}

impl std::error::Error for EncoderError {}

impl Window {
    /// Creates a new window with given dimensions located at the origin.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Window {
            x: 0,
            y: 0,
            w: width,
            h: height,
        }
    }

    /// Moves the window origin to the given origin coordinates.
    #[must_use]
    pub fn at(&self, x: u32, y: u32) -> Window {
        let w = self.w;
        let h = self.h;

        Window { x, y, w, h }
    }

    /// Checks if the window rectangle is inside the canvas rectangle.
    #[must_use]
    fn is_inside(&self, width: u32, height: u32) -> bool {
        self.x + self.w <= width && self.y + self.h <= height
    }

    /// Returns the total number of pixels in the window.
    #[must_use]
    fn len(&self) -> usize {
        (self.w * self.h) as usize
    }
}

impl Canvas {
    /// Returns an iterator over the canvas window image scanlines.
    ///
    /// The iteration starts from the window origin and goes in the positive Y direction.
    /// Each window scanline is represented as a pixel span (`&[Pixel]` slice).
    ///
    /// # Errors
    ///
    /// Returns `None` is the window rectangle origin or dimensions
    /// are out of the canvas bounds.
    #[must_use]
    pub fn window_spans(&self, window: Window) -> Option<WindowSpans<'_>> {
        if !window.is_inside(self.width, self.height) {
            return None;
        }

        let canvas = self;

        // Start iterating from the window origin.
        let scanline = window.y;

        let iter = WindowSpans {
            canvas,
            window,
            scanline,
        };

        Some(iter)
    }

    /// Exports the canvas contents in the requested image format.
    ///
    /// # Errors
    ///
    /// Returns [`EncoderError::NotImplemented`] if the requested image format
    /// is not yet supported.
    #[cfg(not(feature = "png"))]
    pub fn export_image(&self, format: ImageFormat) -> Result<Vec<u8>, EncoderError> {
        // Export the entire canvas.
        let window = Window::new(self.width, self.height);

        match format {
            ImageFormat::RawGamma8Bpp => self.export_raw8bpp(window),
            ImageFormat::RawLinear10BppLE => self.export_raw1xbpp::<10>(window),
            ImageFormat::RawLinear12BppLE => self.export_raw1xbpp::<12>(window),
            _ => Err(EncoderError::NotImplemented),
        }
    }

    /// Exports the canvas window image in the requested image format.
    ///
    /// # Errors
    ///
    /// Returns [`EncoderError::NotImplemented`] if the requested image format
    /// is not yet supported.
    ///
    /// Returns [`EncoderError::BrokenWindow`] if the window rectangle origin
    /// or dimensions are out of the canvas bounds.
    #[cfg(not(feature = "png"))]
    pub fn export_window_image(
        &self,
        window: Window,
        format: ImageFormat,
    ) -> Result<Vec<u8>, EncoderError> {
        if !window.is_inside(self.width, self.height) {
            return Err(EncoderError::BrokenWindow);
        }

        match format {
            ImageFormat::RawGamma8Bpp => self.export_raw8bpp(window),
            ImageFormat::RawLinear10BppLE => self.export_raw1xbpp::<10>(window),
            ImageFormat::RawLinear12BppLE => self.export_raw1xbpp::<12>(window),
            _ => Err(EncoderError::NotImplemented),
        }
    }

    /// Exports the subsampled canvas image in the requested image format.
    ///
    /// The integer subsampling factors in X and Y directions
    /// are passed in `factors`.
    ///
    /// # Errors
    ///
    /// Returns [`EncoderError::NotImplemented`] if the requested image format
    /// is not yet supported.
    #[cfg(not(feature = "png"))]
    pub fn export_subsampled_image(
        &self,
        factors: (u32, u32),
        format: ImageFormat,
    ) -> Result<Vec<u8>, EncoderError> {
        match format {
            ImageFormat::RawGamma8Bpp => self.export_sub_raw8bpp(factors),
            ImageFormat::RawLinear10BppLE => self.export_sub_raw1xbpp::<10>(factors),
            ImageFormat::RawLinear12BppLE => self.export_sub_raw1xbpp::<12>(factors),
            _ => Err(EncoderError::NotImplemented),
        }
    }

    /// Exports the canvas contents in the requested image format.
    ///
    /// # Errors
    ///
    /// Returns [`EncoderError::NotImplemented`] if the requested image format
    /// is not yet supported.
    #[cfg(feature = "png")]
    pub fn export_image(&self, format: ImageFormat) -> Result<Vec<u8>, EncoderError> {
        // Export the entire canvas.
        let window = Window::new(self.width, self.height);

        match format {
            ImageFormat::RawGamma8Bpp => self.export_raw8bpp(window),
            ImageFormat::RawLinear10BppLE => self.export_raw1xbpp::<10>(window),
            ImageFormat::RawLinear12BppLE => self.export_raw1xbpp::<12>(window),
            ImageFormat::PngGamma8Bpp => self.export_png8bpp(window),
            ImageFormat::PngLinear16Bpp => self.export_png16bpp(window),
        }
    }

    /// Exports the canvas window image in the requested image format.
    ///
    /// # Errors
    ///
    /// Returns [`EncoderError::NotImplemented`] if the requested image format
    /// is not yet supported.
    ///
    /// Returns [`EncoderError::BrokenWindow`] if the window rectangle origin
    /// or dimensions are out of the canvas bounds.
    #[cfg(feature = "png")]
    pub fn export_window_image(
        &self,
        window: Window,
        format: ImageFormat,
    ) -> Result<Vec<u8>, EncoderError> {
        if !window.is_inside(self.width, self.height) {
            return Err(EncoderError::BrokenWindow);
        }

        match format {
            ImageFormat::RawGamma8Bpp => self.export_raw8bpp(window),
            ImageFormat::RawLinear10BppLE => self.export_raw1xbpp::<10>(window),
            ImageFormat::RawLinear12BppLE => self.export_raw1xbpp::<12>(window),
            ImageFormat::PngGamma8Bpp => self.export_png8bpp(window),
            ImageFormat::PngLinear16Bpp => self.export_png16bpp(window),
        }
    }

    /// Exports the subsampled canvas image in the requested image format.
    ///
    /// The integer subsampling factors in X and Y directions
    /// are passed in `factors`.
    ///
    /// # Errors
    ///
    /// Returns [`EncoderError::NotImplemented`] if the requested image format
    /// is not yet supported.
    #[cfg(feature = "png")]
    pub fn export_subsampled_image(
        &self,
        factors: (u32, u32),
        format: ImageFormat,
    ) -> Result<Vec<u8>, EncoderError> {
        match format {
            ImageFormat::RawGamma8Bpp => self.export_sub_raw8bpp(factors),
            ImageFormat::RawLinear10BppLE => self.export_sub_raw1xbpp::<10>(factors),
            ImageFormat::RawLinear12BppLE => self.export_sub_raw1xbpp::<12>(factors),
            ImageFormat::PngGamma8Bpp => self.export_sub_png8bpp(factors),
            ImageFormat::PngLinear16Bpp => self.export_sub_png16bpp(factors),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SpotShape;

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

    #[test]
    fn get_window_spans() {
        let mut c = Canvas::new(100, 100);

        c.add_spot((50.75, 50.5), SpotShape::default(), 1.0);
        c.draw();

        let wnd1 = Window::new(4, 3).at(50, 50);

        let mut vec = Vec::new();
        for span in c.window_spans(wnd1).unwrap() {
            vec.extend_from_slice(span);
        }

        assert_eq!(
            vec,
            [542, 18256, 1146, 0, 542, 18256, 1146, 0, 202, 744, 0, 0]
        );

        let wnd2 = wnd1.at(48, 51);

        vec.clear();
        for span in c.window_spans(wnd2).unwrap() {
            vec.extend_from_slice(span);
        }

        assert_eq!(vec, [0, 3, 542, 18256, 0, 0, 202, 744, 0, 0, 0, 0]);
    }

    #[test]
    fn broken_windows() {
        let c = Canvas::new(100, 100);

        let wnd1 = Window::new(4, 100).at(50, 50);
        assert!(c.window_spans(wnd1).is_none());

        let wnd2 = Window::new(4, 5).at(100, 100);
        assert!(c.window_spans(wnd2).is_none());

        let wnd3 = Window::new(1, 1).at(100, 100);
        assert!(c.window_spans(wnd3).is_none());

        let wnd4 = Window::new(0, 0).at(100, 100);
        let mut spans = c.window_spans(wnd4).unwrap();
        assert_eq!(spans.len(), 0);
        assert!(spans.next().is_none());
    }
}
