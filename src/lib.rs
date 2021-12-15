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
//! It is possible to define a custom world coordinates to canvas coordinates
//! transformation, which affects all spots.
//!
//! ```
//! use planetarium::{Canvas, SpotShape, Transform};
//!
//! // Draw on a square 256x256 pixel canvas.
//! let mut c = Canvas::new(256, 256);
//!
//! // Define an elliptic spot shape with diffraction radii of 2.5 x 1.5 pixels
//! // rotated by 45 degrees counter-clockwise.
//! let shape1 = SpotShape::default().stretch(2.5, 1.5).rotate(45.0);
//!
//! // Define an elliptic spot shape by a 2x2 linear transform matrix.
//! let shape2 = SpotShape::from([[2.0, -0.5], [1.5, 3.0]]);
//!
//! // Add some spots at random positions with varying shape size
//! // and peak intensity.
//! let spot1 = c.add_spot((100.3, 130.8), shape1, 0.5);
//! let spot2 = c.add_spot((80.6, 200.2), shape2, 0.9);
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
//!
//! // Apply a custom world coordinates to canvas coordinates transformation.
//! c.set_view_transform(Transform::default().translate((13.7, -20.3)));
//!
//! // Query the resulting spot coordinates on the canvas after
//! // the view coordinate transformation.
//! assert_eq!(c.spot_position(spot1), Some((100.3 - 34.2 + 13.7, 130.8 + 12.6 - 20.3)));
//! assert_eq!(c.spot_position(spot2), Some((80.6 + 114.2 + 13.7, 200.2 - 73.3 - 20.3)));
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
mod export;
mod gamma;
mod pattern;

pub use crate::export::{EncoderError, ImageFormat, Window, WindowSpans};

use crate::gamma::GammaCurve8;
use crate::pattern::AiryPattern;

/// Image pixel value type: 16-bit pixels
pub type Pixel = u16;

/// 2D point coordinates: `(X, Y)`
pub type Point = (f32, f32);

/// 2D vector coordinates: `(X, Y)`
pub type Vector = (f32, f32);

/// 2x2 matrix: `[[a11, a12], [a21, a22]]`
pub type Matrix = [[f32; 2]; 2];

/// 2x3 matrix: `[[a11, a12, a13], [a21, a22, a23]]`
pub type Matrix23 = [[f32; 3]; 2];

/// Spot shape definition matrix
///
/// A unit sized circular spot is scaled
/// using the 2x2 transform matrix.
///
/// Basic operations
/// ----------------
///
/// ```
/// use planetarium::SpotShape;
///
/// // Create a unit-sized circular spot.
/// let s1 = SpotShape::default();
///
/// // Upscale 2x
/// let s2 = s1.scale(2.0);
///
/// // Stretch by 1.5 in the X direction and rotate clockwise by 45 degrees.
/// let s3 = s2.stretch(1.5, 1.0).rotate(-45.0);
/// ```
///
/// Conversions
/// -----------
///
/// ```
/// # use planetarium::SpotShape;
/// // From a scalar size factor
/// let s1 = SpotShape::from(2.0);
///
/// // From X-direction and Y-direction sizes
/// let s2 = SpotShape::from((2.0, 1.5));
///
/// // From a 2x2 linear coordinate transform matrix
/// let s3 = SpotShape::from([[1.5, -0.5], [0.5, 2.5]]);
/// ```
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

/// 2D affine transformation definition matrix
///
/// Contains a 2x3 linear transform matrix to be applied
/// to homogenous coordinates internally.
///
/// Basic operations
/// ----------------
///
/// ```
/// use planetarium::Transform;
///
/// let t1 = Transform::default().translate((-10.0, 12.0));
/// let t2 = t1.scale(1.5).rotate(-45.0);
/// let t3 = t1.stretch(1.5, 3.0).compose(t2);
/// ```
///
/// Conversions
/// -----------
///
/// ```
/// # use planetarium::Transform;
/// // From a scalar factor
/// let t1 = Transform::from(2.0);
///
/// // From a translation vector
/// let t2 = Transform::from((2.0, -3.5));
///
/// // From a 2x2 linear coordinate transform matrix
/// let t3 = Transform::from([[1.5, -0.5], [0.5, 2.5]]);
///
/// // From a 2x3 affine coordinate transform matrix
/// let t4 = Transform::from([[1.5, -0.5, 4.5], [0.5, 2.5, 0.0]]);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// a11 - X scale
    pub xx: f32,
    /// a12 - XY shear
    pub xy: f32,
    /// a21 - YX shear
    pub yx: f32,
    /// a22 - Y scale
    pub yy: f32,

    /// a13 - X translation
    pub tx: f32,
    /// a23 - Y translation
    pub ty: f32,
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

/// Opaque light spots drawing canvas object
///
/// Generates the synthesized image containing multiple light spots.
///
/// Basic operations
/// ----------------
///
/// ```
/// use planetarium::{Canvas, Transform};
///
/// // Draw on a square 256x256 pixel canvas.
/// let mut c = Canvas::new(256, 256);
///
/// // Set the canvas background pixel value.
/// c.set_background(100);
///
/// // Set the global brightness level.
/// c.set_brightness(1.5);
///
/// // Set the view coordinate transformation: linear shift by [14.5 , -5.0].
/// c.set_view_transform(Transform::default().translate((14.5, -5.0)));
///
/// // Clear the canvas.
/// c.clear();
///
/// // Clear the canvas and paint the light spots.
/// c.draw();
///
/// // Access the rendered image data as a linear pixel array.
/// let image_pixbuf = c.pixels();
///
/// // Get pixel at x = 100, y = 200.
/// let (x, y) = (100, 200);
/// let (image_width, image_height) = c.dimensions();
/// let val_x_y = image_pixbuf[(y * image_width + x) as usize];
/// ```
pub struct Canvas {
    /// Canvas width in pixels
    width: u32,

    /// Canvas height in pixels
    height: u32,

    /// Background light level
    background: Pixel,

    /// Light spot draw list
    spots: Vec<SpotRec>,

    /// View transform matrix
    transform: Transform,

    /// Global spot brightness factor
    brightness: f32,

    /// Image pixel buffer
    pixbuf: Vec<Pixel>,

    /// Spot pattern lookup table
    pattern: AiryPattern,

    /// sRBG compression gamma curve LUT
    gamma_curve: GammaCurve8,
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

impl From<f32> for SpotShape {
    fn from(size: f32) -> Self {
        Self::default().scale(size)
    }
}

impl From<(f32, f32)> for SpotShape {
    fn from(kxy: (f32, f32)) -> Self {
        Self::default().stretch(kxy.0, kxy.1)
    }
}

impl From<Matrix> for SpotShape {
    fn from(mat: Matrix) -> Self {
        SpotShape {
            xx: mat[0][0],
            xy: mat[0][1],
            yx: mat[1][0],
            yy: mat[1][1],
        }
    }
}

impl std::fmt::Display for SpotShape {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[[{}, {}], [{}, {}]]",
            self.xx, self.xy, self.yx, self.yy
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            xx: 1.0,
            xy: 0.0,
            yx: 0.0,
            yy: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }
}

impl From<f32> for Transform {
    fn from(k: f32) -> Self {
        Self::default().scale(k)
    }
}

impl From<Vector> for Transform {
    fn from(shift: Vector) -> Self {
        Self::default().translate(shift)
    }
}

impl From<Matrix> for Transform {
    fn from(mat: Matrix) -> Self {
        Transform {
            xx: mat[0][0],
            xy: mat[0][1],
            yx: mat[1][0],
            yy: mat[1][1],
            tx: 0.0,
            ty: 0.0,
        }
    }
}

impl From<Matrix23> for Transform {
    fn from(mat: Matrix23) -> Self {
        Transform {
            xx: mat[0][0],
            xy: mat[0][1],
            yx: mat[1][0],
            yy: mat[1][1],
            tx: mat[0][2],
            ty: mat[1][2],
        }
    }
}

impl std::fmt::Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[[{}, {}, {}], [{}, {}, {}]]",
            self.xx, self.xy, self.tx, self.yx, self.yy, self.ty
        )
    }
}

impl SpotShape {
    /// Linearly scales the spot shape by a single scalar factor.
    pub fn scale(&self, k: f32) -> SpotShape {
        let xx = k * self.xx;
        let xy = k * self.xy;
        let yx = k * self.yx;
        let yy = k * self.yy;

        SpotShape { xx, xy, yx, yy }
    }

    /// Linearly stretches the spot shape in X and Y directions.
    pub fn stretch(&self, kx: f32, ky: f32) -> SpotShape {
        let xx = kx * self.xx;
        let xy = kx * self.xy;
        let yx = ky * self.yx;
        let yy = ky * self.yy;

        SpotShape { xx, xy, yx, yy }
    }

    /// Rotates the spot shape counter-clockwise by `phi` degrees.
    pub fn rotate(&self, phi: f32) -> SpotShape {
        let phi_rad = (std::f32::consts::PI / 180.0) * phi;

        let (s, c) = phi_rad.sin_cos();

        let xx = c * self.xx - s * self.yx;
        let yx = c * self.yx + s * self.xx;
        let xy = c * self.xy - s * self.yy;
        let yy = c * self.yy + s * self.xy;

        SpotShape { xx, xy, yx, yy }
    }
}

impl Transform {
    /// Linearly translates the output coordinates by a shift vector.
    pub fn translate(&self, shift: Vector) -> Transform {
        let mut transform = *self;

        transform.tx += shift.0;
        transform.ty += shift.1;

        transform
    }

    /// Linearly scales the coordinates by a single scalar factor.
    pub fn scale(&self, k: f32) -> Transform {
        let xx = k * self.xx;
        let xy = k * self.xy;
        let yx = k * self.yx;
        let yy = k * self.yy;
        let tx = k * self.tx;
        let ty = k * self.ty;

        Transform {
            xx,
            xy,
            yx,
            yy,
            tx,
            ty,
        }
    }

    /// Linearly stretches the coordinates in X and Y directions.
    pub fn stretch(&self, kx: f32, ky: f32) -> Transform {
        let xx = kx * self.xx;
        let xy = kx * self.xy;
        let yx = ky * self.yx;
        let yy = ky * self.yy;
        let tx = kx * self.tx;
        let ty = ky * self.ty;

        Transform {
            xx,
            xy,
            yx,
            yy,
            tx,
            ty,
        }
    }

    /// Rotates the coordinates counter-clockwise by `phi` degrees.
    pub fn rotate(&self, phi: f32) -> Transform {
        let phi_rad = (std::f32::consts::PI / 180.0) * phi;

        let (s, c) = phi_rad.sin_cos();

        let xx = c * self.xx - s * self.yx;
        let yx = c * self.yx + s * self.xx;
        let xy = c * self.xy - s * self.yy;
        let yy = c * self.yy + s * self.xy;
        let tx = c * self.tx - s * self.ty;
        let ty = c * self.ty + s * self.tx;

        Transform {
            xx,
            xy,
            yx,
            yy,
            tx,
            ty,
        }
    }

    /// Composes the coordinate transformation with an outer transformation.
    ///
    /// In the matrix multiplication form: `[t][self]`
    pub fn compose(&self, t: Transform) -> Transform {
        let xx = self.xx * t.xx + self.yx * t.xy;
        let xy = self.xy * t.xx + self.yy * t.xy;
        let yx = self.yx * t.yy + self.xx * t.yx;
        let yy = self.yy * t.yy + self.xy * t.yx;
        let tx = self.tx * t.xx + self.ty * t.xy + t.tx;
        let ty = self.ty * t.yy + self.tx * t.yx + t.ty;

        Transform {
            xx,
            xy,
            yx,
            yy,
            tx,
            ty,
        }
    }

    /// Transforms 2D point coordinates using the affine transformation matrix.
    fn apply(&self, p: Point) -> Point {
        let x = p.0 * self.xx + p.1 * self.xy + self.tx;
        let y = p.1 * self.yy + p.0 * self.yx + self.ty;

        (x, y)
    }
}

impl Canvas {
    /// Creates a new clear canvas to render light spots on.
    pub fn new(width: u32, height: u32) -> Self {
        let background = 0;
        let spots = Vec::with_capacity(8);
        let transform = Transform::default();
        let brightness = 1.0;
        let pixbuf = vec![0; (width * height) as usize];
        let pattern = AiryPattern::new();
        let gamma_curve = GammaCurve8::new();

        Canvas {
            width,
            height,
            background,
            spots,
            transform,
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
    /// shifted by the variable spot offset vector and transformed using the view
    /// coordinate transformation.
    pub fn spot_position(&self, spot: SpotId) -> Option<Point> {
        let view_transform = |s: &SpotRec| {
            let world_pos = ((s.position.0 + s.offset.0), (s.position.1 + s.offset.1));
            self.transform.apply(world_pos)
        };

        self.spots.get(spot).map(view_transform)
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

    /// Sets the world coordinates to canvas coordinates transformation.
    ///
    /// The light spot coordinates are defined in the world coordinate system only.
    pub fn set_view_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    /// Sets the global brightness level (light spot intensity adjustment).
    pub fn set_brightness(&mut self, brightness: f32) {
        self.brightness = brightness;
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
    fn create_shapes() {
        let s1 = SpotShape::default();
        assert_eq!(s1.to_string(), "[[1, 0], [0, 1]]");

        let s2 = s1.scale(2.0);
        assert_eq!(s2.to_string(), "[[2, 0], [0, 2]]");

        let s3 = s2.stretch(2.0, 3.0);
        assert_eq!(s3.to_string(), "[[4, 0], [0, 6]]");

        let s4 = s3.rotate(90.0);
        assert!(s4.xx.abs() < 1e-4, "xx = {}", s4.xx);
        assert!((s4.xy - (-6.0)).abs() < 1e-4, "xy = {}", s4.xy);
        assert!((s4.yx - 4.0).abs() < 1e-4, "yx = {}", s4.yx);
        assert!(s4.yy.abs() < 1e-4, "yy = {}", s4.yy);
    }

    #[test]
    fn convert_shapes() {
        let s1 = SpotShape::from(1.0);
        assert_eq!(s1.to_string(), "[[1, 0], [0, 1]]");

        let s2: SpotShape = 2.0.into();
        assert_eq!(s2.to_string(), "[[2, 0], [0, 2]]");

        let s3 = SpotShape::from((2.0, 3.0)).scale(2.0);
        assert_eq!(s3.to_string(), "[[4, 0], [0, 6]]");

        let s4 = SpotShape::from([[1.0, 2.0], [3.0, 4.0]]);
        assert_eq!(s4.to_string(), "[[1, 2], [3, 4]]");
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
    fn create_transform() {
        let t1 = Transform::default().translate((3.5, -4.25));
        assert_eq!(t1.to_string(), "[[1, 0, 3.5], [0, 1, -4.25]]");

        let t2 = Transform::default().scale(3.5).translate((1.0, 2.0));
        assert_eq!(t2.to_string(), "[[3.5, 0, 1], [0, 3.5, 2]]");

        let t3 = t2.stretch(3.5, 4.0);
        assert_eq!(t3.to_string(), "[[12.25, 0, 3.5], [0, 14, 8]]");

        let t4 = t3.compose(t1);
        assert_eq!(t4.to_string(), "[[12.25, 0, 7], [0, 14, 3.75]]");

        let t5 = t4.compose(t2);
        assert_eq!(t5.to_string(), "[[42.875, 0, 25.5], [0, 49, 15.125]]");

        let p = t5.apply((1.0, 1.0));
        assert_eq!(p, (68.375, 64.125));
    }

    #[test]
    fn convert_transforms() {
        let t1 = Transform::from(1.0);
        assert_eq!(t1.to_string(), "[[1, 0, 0], [0, 1, 0]]");

        let t2: Transform = 2.0.into();
        assert_eq!(t2.to_string(), "[[2, 0, 0], [0, 2, 0]]");

        let t3 = Transform::from((2.0, 3.0)).scale(2.0);
        assert_eq!(t3.to_string(), "[[2, 0, 4], [0, 2, 6]]");

        let t4 = Transform::from([[1.0, 2.0], [3.0, 4.0]]);
        assert_eq!(t4.to_string(), "[[1, 2, 0], [3, 4, 0]]");

        let t5 = Transform::from([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        assert_eq!(t5.to_string(), "[[1, 2, 3], [4, 5, 6]]");
    }

    #[test]
    fn view_transform() {
        let shape = SpotShape::default();
        let mut c = Canvas::new(16, 16);

        let spot1 = c.add_spot((1.1, 4.3), shape, 0.5);
        let spot2 = c.add_spot((4.6, 7.2), shape, 0.4);

        assert_eq!(c.spot_position(spot1), Some((1.1, 4.3)));

        let transform = Transform::default().translate((3.2, -4.8));
        c.set_view_transform(transform);

        assert_eq!(c.spot_position(spot1), Some((1.1 + 3.2, 4.3 - 4.8)));
        assert_eq!(c.spot_position(spot2), Some((4.6 + 3.2, 7.2 - 4.8)));
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
