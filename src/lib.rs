//! Planetarium
//! ===========
//!
//! Sub-pixel precision light spot rendering library for astronomy
//! and video tracking applications.

mod draw;
mod pattern;

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
    pattern_lut: Vec<f32>,

    /// Pattern LUT index scaling factor
    pattern_scale: f32,
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

impl Canvas {
    /// Creates a new clear canvas to render light spots on.
    pub fn new(width: u32, height: u32) -> Self {
        let background = 0;
        let spots = Vec::with_capacity(8);
        let brightness = 1.0;
        let pixbuf = vec![0; (width * height) as usize];
        let (pattern_lut, pattern_scale) = Self::build_pattern_lut();

        Canvas {
            width,
            height,
            background,
            spots,
            brightness,
            pixbuf,
            pattern_lut,
            pattern_scale,
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

        c.set_spot_offset(spot1, (-3.2, 4.2));
        c.set_spot_illumination(spot2, 1.3);

        // NOP
        c.set_spot_offset(55, (1.1, 1.2));

        // NOP
        c.set_spot_illumination(33, 0.0);
    }
}
