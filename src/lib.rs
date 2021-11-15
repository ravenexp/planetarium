//! Planetarium
//! ===========
//!
//! Sub-pixel precision light spot rendering library for astronomy
//! and video tracking applications.
//!
//! Example usage
//! -------------
//!
//! ```
//! use planetarium::{Canvas, SpotShape};
//!
//! // Draw on a square 256x256 pixel canvas.
//! let mut c = Canvas::new(256, 256);
//!
//! // Define a round spot shape with diffraction radius of 2.5 pixels.
//! let shape = SpotShape::default().scale(2.5);
//!
//! // Add some spots at random positions with varying shape size
//! // and peak intensity.
//! let spot1 = c.add_spot((100.3, 130.8), shape, 0.5);
//! let spot2 = c.add_spot((80.6, 200.2), shape.scale(0.5), 0.9);
//!
//! // Note: Out of range position coordinates and peak intensities are fine.
//! //       The resulting spot image is clipped into the canvas rectangle.
//! //       Peak intensity > 1.0 leads to saturation to the maximum pixel value.
//! let spot3 = c.add_spot((256.1, 3.5), shape.scale(10.0), 1.1);
//!
//! // Set the canvas background pixel value.
//! c.set_background(100);
//!
//! // Clear the canvas and paint the light spots.
//! c.draw();
//!
//! // Access the rendered image data as a linear pixel array.
//! let image_pixbuf = c.pixels();
//!
//! // Get pixel at x = 100, y = 200.
//! let (x, y) = (100, 200);
//! let (image_width, image_height) = c.dimensions();
//! let val_x_y = image_pixbuf[(y * image_width + x) as usize];
//! ```
//!
//! Light spot parameters adjustment
//! --------------------------------
//!
//! Some of the light spot parameters like coordinates and peak intensity
//! can be adjusted after the spot has been added to the canvas.
//!
//! The spot position coordinates can be changed by adding an offset vector
//! and the peak intensity can be adjusted by multiplying with a spot
//! illumination factor.
//!
//! ```
//! use planetarium::{Canvas, SpotShape};
//!
//! // Draw on a square 256x256 pixel canvas.
//! let mut c = Canvas::new(256, 256);
//!
//! // Define a round spot shape with diffraction radius of 2.5 pixels.
//! let shape = SpotShape::default().scale(2.5);
//!
//! // Add some spots at random positions with varying shape size
//! // and peak intensity.
//! let spot1 = c.add_spot((100.3, 130.8), shape, 0.5);
//! let spot2 = c.add_spot((80.6, 200.2), shape.scale(0.5), 0.9);
//!
//! // Shift the rendered spot positions by applying the relative offset vectors.
//! // The intrinsic spot position coordinates are immutable.
//! c.set_spot_offset(spot1, (-34.2, 12.6));
//! c.set_spot_offset(spot2, (114.2, -73.3));
//!
//! // Adjust the rendered spot peak intensity by applying the spot illumination factors.
//! // The intrinsic spot intensities are immutable.
//! c.set_spot_illumination(spot1, 1.2);
//! c.set_spot_illumination(spot2, 0.7);
//!
//! // Query the resulting spot coordinates on the canvas.
//! assert_eq!(c.spot_position(spot1), Some((100.3 - 34.2, 130.8 + 12.6)));
//! assert_eq!(c.spot_position(spot2), Some((80.6 + 114.2, 200.2 - 73.3)));
//!
//! // Query the resulting peak spot intensities.
//! assert_eq!(c.spot_intensity(spot1), Some(0.5 * 1.2));
//! assert_eq!(c.spot_intensity(spot2), Some(0.9 * 0.7));
//! ```
//!
//! Canvas image export
//! -------------------
//!
//! The `Canvas` object supports image export to RAW and PNG file formats.
//! Both 8-bit and 16-bit PNG sample formats are supported.
//! Export to PNG formats requires the default `png` feature to be enabled.
//!
//! ### Example RAW image export code
//!
//! ```
//! use planetarium::{Canvas, ImageFormat};
//!
//! let mut c = Canvas::new(256, 256);
//!
//! c.set_background(1000);
//! c.clear();
//!
//! // Export to a 8-bit gamma-compressed grayscale RAW image.
//! let raw_8bpp_bytes = c.export_image(ImageFormat::RawGamma8Bpp).unwrap();
//!
//! // Export to a 10-bit linear light grayscale little-endian RAW image.
//! let raw_10bpp_bytes = c.export_image(ImageFormat::RawLinear10BppLE).unwrap();
//!
//! // Export to a 12-bit gamma-compressed grayscale little-endian RAW image.
//! let raw_12bpp_bytes = c.export_image(ImageFormat::RawLinear12BppLE).unwrap();
//! ```
//!
//! ### Example PNG export code
//!
//! ```
//! use planetarium::{Canvas, ImageFormat};
//!
//! let mut c = Canvas::new(256, 256);
//!
//! c.set_background(1000);
//! c.clear();
//!
//! #[cfg(features = "png")]
//! // Export to a 8-bit gamma-compressed grayscale PNG image.
//! let png_8bpp_bytes = c.export_image(ImageFormat::PngGamma8Bpp).unwrap();
//!
//! #[cfg(features = "png")]
//! // Export to a 16-bit linear light grayscale PNG image.
//! let png_16bpp_bytes = c.export_image(ImageFormat::PngLinear16Bpp).unwrap();
//! ```

mod draw;
mod gamma;
mod pattern;
mod raw;

#[cfg(feature = "png")]
mod png;

use crate::gamma::GammaCurve8;
use crate::pattern::AiryPattern;

/// Image pixel value type: 16-bit pixels
pub type Pixel = u16;

/// 2D point coordinates: `(X, Y)`
pub type Point = (f32, f32);

/// 2D vector coordinates: `(X, Y)`
pub type Vector = (f32, f32);

/// Spot shape definition matrix
///
/// A unit sized circular spot is scaled
/// using the 2x2 transform matrix.
#[derive(Debug, Clone, Copy)]
pub struct SpotShape {
    /// a11 - X dimension
    pub xx: f32,
    /// a12 - XY skew
    pub xy: f32,
    /// a21 - YX skew
    pub yx: f32,
    /// a22 - Y dimension
    pub yy: f32,
}

/// Light spot descriptor type
pub type SpotId = usize;

/// Light spot rendering parameters
#[derive(Debug, Clone, Copy)]
struct SpotRec {
    /// Ligth spot centroid position
    position: Point,

    /// Relative spot position offset
    offset: Vector,

    /// Relative peak intensity
    intensity: f32,

    /// Illumination based spot intensity factor
    illumination: f32,

    /// Spot shape definition matrix
    shape: SpotShape,

    /// Inverted spot shape matrix (cached)
    shape_inv: SpotShape,
}

/// Generates the synthesized image containing multiple light spots
pub struct Canvas {
    /// Canvas width in pixels
    width: u32,

    /// Canvas height in pixels
    height: u32,

    /// Background light level
    background: Pixel,

    /// Light spot draw list
    spots: Vec<SpotRec>,

    /// Global spot brightness factor
    brightness: f32,

    /// Image pixel buffer
    pixbuf: Vec<Pixel>,

    /// Spot pattern lookup table
    pattern: AiryPattern,

    /// sRBG compression gamma curve LUT
    gamma_curve: GammaCurve8,
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

impl Default for SpotShape {
    fn default() -> Self {
        SpotShape {
            xx: 1.0,
            xy: 0.0,
            yx: 0.0,
            yy: 1.0,
        }
    }
}

impl std::fmt::Display for EncoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // FIXME: Put full length error descriptions here.
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EncoderError {}

impl SpotShape {
    /// Linearly scales the spot shape by a single scalar factor.
    pub fn scale(&self, k: f32) -> SpotShape {
        let xx = k * self.xx;
        let xy = k * self.xy;
        let yx = k * self.yx;
        let yy = k * self.yy;

        SpotShape { xx, xy, yx, yy }
    }
}

impl Canvas {
    /// Creates a new clear canvas to render light spots on.
    pub fn new(width: u32, height: u32) -> Self {
        let background = 0;
        let spots = Vec::with_capacity(8);
        let brightness = 1.0;
        let pixbuf = vec![0; (width * height) as usize];
        let pattern = AiryPattern::new();
        let gamma_curve = GammaCurve8::new();

        Canvas {
            width,
            height,
            background,
            spots,
            brightness,
            pixbuf,
            pattern,
            gamma_curve,
        }
    }

    /// Creates a new light spot on the canvas.
    pub fn add_spot(&mut self, position: Point, shape: SpotShape, intensity: f32) -> SpotId {
        // Initialize with the defaults
        let offset = (0.0, 0.0);
        let illumination = 1.0;

        // Pre-compute and cache the inverted spot shape matrix
        // used by the rasterizer code.
        let shape_inv = shape.invert();

        let spot = SpotRec {
            position,
            offset,
            shape,
            intensity,
            illumination,
            shape_inv,
        };

        let id = self.spots.len();
        self.spots.push(spot);
        id
    }

    /// Calculates the canvas coordinates of the light spot.
    ///
    /// The canvas coordinates are calculated as the immutable spot position coordinates
    /// shifted by the variable spot offset vector and transformed using the canvas
    /// world transform.
    pub fn spot_position(&self, spot: SpotId) -> Option<Point> {
        self.spots
            .get(spot)
            .map(|s| ((s.position.0 + s.offset.0), (s.position.1 + s.offset.1)))
    }

    /// Calculates the effective peak intensity of the light spot.
    ///
    /// The effective peak intensity is calculated as the product of the immutable spot
    /// intensity factor, the variable spot illumination factor
    /// and the global brightness level.
    pub fn spot_intensity(&self, spot: SpotId) -> Option<f32> {
        self.spots
            .get(spot)
            .map(|s| s.intensity * s.illumination * self.brightness)
    }

    /// Sets the internal light spot position offset vector.
    ///
    /// The position offset vector is added to the immutable spot position
    /// to calculate the spot rendering coordinates on the canvas.
    pub fn set_spot_offset(&mut self, spot: SpotId, offset: Vector) {
        if let Some(s) = self.spots.get_mut(spot) {
            s.offset = offset;
        }
    }

    /// Sets the internal light spot illumination state.
    ///
    /// The spot illumination factor is multiplied with the immutable spot
    /// intensity factor to calculate the rendered peak intensity.
    pub fn set_spot_illumination(&mut self, spot: SpotId, illumination: f32) {
        if let Some(s) = self.spots.get_mut(spot) {
            s.illumination = illumination;
        }
    }

    /// Clears the canvas image (fills with background pixels).
    pub fn clear(&mut self) {
        self.pixbuf.fill(self.background)
    }

    /// Draws the light spots onto the canvas image.
    pub fn draw(&mut self) {
        // Always clear the canvas first to avoid unintended overdraw.
        self.clear();

        if self.brightness <= 0.0 {
            return;
        }

        // `self.spots` can not be borrowed for `draw_spot()`
        for spot_id in 0..self.spots.len() {
            self.draw_spot(spot_id)
        }
    }

    /// Returns the rendered image pixels buffer.
    pub fn pixels(&self) -> &[Pixel] {
        &self.pixbuf
    }

    /// Returns the canvas dimensions as `(width, height)`.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Sets the background light level (dark pixel value).
    pub fn set_background(&mut self, level: Pixel) {
        self.background = level;
    }

    /// Sets the global brightness level (light spot intensity adjustment).
    pub fn set_brightness(&mut self, brightness: f32) {
        self.brightness = brightness;
    }

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

    #[test]
    fn create_canvas() {
        let w = 16;
        let h = 16;

        let c = Canvas::new(w, h);
        assert_eq!(c.width, w);
        assert_eq!(c.height, h);

        let sz = c.pixels().len();
        assert_eq!(sz, (w * h) as usize);

        let dim = c.dimensions();
        assert_eq!(dim, (w, h));
    }

    #[test]
    fn add_spots() {
        let shape = SpotShape::default();
        let mut c = Canvas::new(16, 16);

        let spot1 = c.add_spot((1.1, 4.3), shape, 0.5);
        let spot2 = c.add_spot((4.6, 7.2), shape, 0.4);

        assert_eq!(spot1, 0);
        assert_eq!(spot2, 1);
    }

    #[test]
    fn clear_canvas() {
        let mut c = Canvas::new(16, 16);

        assert_eq!(c.pixels()[0], 0);

        c.set_background(100);
        c.clear();

        assert_eq!(c.pixels()[0], 100);

        c.set_background(200);
        c.draw();

        assert_eq!(c.pixels()[0], 200);
    }

    #[test]
    fn move_spots() {
        let shape = SpotShape::default();
        let mut c = Canvas::new(16, 16);

        let spot1 = c.add_spot((1.1, 4.3), shape, 0.5);
        let spot2 = c.add_spot((4.6, 7.2), shape, 0.4);

        assert_eq!(c.spot_position(spot1), Some((1.1, 4.3)));
        assert_eq!(c.spot_intensity(spot2), Some(0.4));

        c.set_spot_offset(spot1, (-3.2, 4.2));
        c.set_spot_illumination(spot2, 1.3);

        assert_eq!(c.spot_position(spot1), Some((1.1 - 3.2, 4.3 + 4.2)));
        assert_eq!(c.spot_intensity(spot2), Some(0.4 * 1.3));

        // NOP
        c.set_spot_offset(55, (1.1, 1.2));

        // NOP
        c.set_spot_illumination(33, 0.0);
    }

    #[test]
    fn draw_spots() {
        let shape = SpotShape::default().scale(2.5);
        let mut c = Canvas::new(32, 32);

        let spot1 = c.add_spot((1.1, 4.3), shape, 0.5);
        let spot2 = c.add_spot((4.6, 7.2), shape, 0.9);
        let spot3 = c.add_spot((17.3, 25.8), shape, 1.2);
        let spot4 = c.add_spot((30.6, 10.1), shape, 0.7);

        c.set_background(1000);
        c.draw();

        assert_eq!(c.pixels()[32 * 4 + 1], 31823);
        assert_eq!(c.pixels()[32 * 7 + 5], 53389);
        assert_eq!(c.pixels()[32 * 26 + 17], 65535);
        assert_eq!(c.pixels()[32 * 10 + 30], 37774);

        c.set_spot_offset(spot3, (-13.2, 6.2));

        for s in [spot1, spot2, spot3, spot4] {
            c.set_spot_illumination(s, 1.5);
        }

        c.draw();

        assert_eq!(c.pixels()[32 * 4 + 1], 47235);
        assert_eq!(c.pixels()[32 * 7 + 5], 65535);
        assert_eq!(c.pixels()[32 * 26 + 17], 1000);
        assert_eq!(c.pixels()[32 * 10 + 30], 56161);

        // new spot3 location
        assert_eq!(c.pixels()[32 * 31 + 3], 28020);
    }
}
