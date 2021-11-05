//! Planetarium
//! ===========
//!
//! Private light spot image drawing routines
//! -----------------------------------------
//!
//! Contains private types and implementations of private methods
//! for the existing public types.

use super::{Canvas, Pixel, Point, SpotId, SpotRec, SpotShape, Vector};
use crate::pattern::{J1_ZERO1, J1_ZERO2};

impl SpotShape {
    /// Fudge factor for the effective spot radius estimation
    ///
    /// The unit radius is the radius of the Airy disc at the first minumum,
    /// also known as the diffraction radius.
    /// The effective (rasterized) spot radius is arbitrarily chosen as
    /// the radius of the second Airy disc minumum.
    const EFFECTIVE_RADIUS_FACTOR: f32 = J1_ZERO2 / J1_ZERO1;

    /// Calculates the effective radius of the spot image
    /// projected onto the coordinate axes as XY components.
    fn effective_radius_xy(&self) -> (f32, f32) {
        // Rx = F*sqrt(a11^2 + a12^2), Ry = F*sqrt(a22^2 + a21^2))
        (
            Self::EFFECTIVE_RADIUS_FACTOR * self.xx.hypot(self.xy),
            Self::EFFECTIVE_RADIUS_FACTOR * self.yy.hypot(self.yx),
        )
    }

    /// Inverts the shape definition matrix.
    ///
    /// Returns the inverted matrix.
    pub(super) fn invert(&self) -> SpotShape {
        let det = self.xx * self.yy - self.xy * self.yx;

        // Bail on (almost) singular matrices in debug builds,
        // fall back to the unit shape in releases.
        if det.abs() < 0.01 {
            debug_assert!(false, "Singular shape matrix: {:?}", self);
            return SpotShape::default();
        }

        let inv_det = det.recip();

        let xx = inv_det * self.yy;
        let yy = inv_det * self.xx;
        let xy = inv_det * -self.xy;
        let yx = inv_det * -self.yx;

        SpotShape { xx, xy, yx, yy }
    }

    /// Transforms a 2D vector using the shape definition matrix.
    ///
    /// Returns the transformed vector.
    fn apply(&self, vec: Vector) -> Vector {
        let x = vec.0 * self.xx + vec.1 * self.xy;
        let y = vec.1 * self.yy + vec.0 * self.yx;

        (x, y)
    }
}

/// Spot bounding box coordinates in pixels
#[derive(Debug, Clone, Copy)]
struct BoundingBox {
    /// Top left corner X (inclusive)
    x0: u32,
    /// Top left corner Y (inclusive)
    y0: u32,
    /// Bottom right corner X (exclusive)
    x1: u32,
    /// Bottom right corner Y (exclusive)
    y1: u32,
}

impl BoundingBox {
    /// Calculates the bounding box for a light spot.
    ///
    /// Clips to box dimensions to the underlying canvas size.
    fn new(spot: &SpotRec, width: u32, height: u32) -> Self {
        let (rx, ry) = spot.shape.effective_radius_xy();
        let (px, py) = spot.position();
        let (w, h) = (width as i32, height as i32);

        let x0 = ((px - rx).floor() as i32).max(0).min(w) as u32;
        let y0 = ((py - ry).floor() as i32).max(0).min(h) as u32;
        let x1 = ((px + rx).ceil() as i32).max(0).min(w) as u32;
        let y1 = ((py + ry).ceil() as i32).max(0).min(h) as u32;

        BoundingBox { x0, y0, x1, y1 }
    }

    /// Checks if the bounding box is contains no pixels.
    fn is_empty(&self) -> bool {
        self.x0 == self.x1 || self.y0 == self.y1
    }
}

impl SpotRec {
    /// Calculates the effective spot position by taking into account
    /// the associated position offset vector.
    fn position(&self) -> Point {
        (
            (self.position.0 + self.offset.0),
            (self.position.1 + self.offset.1),
        )
    }
}

impl Canvas {
    /// Draws a single light spot image on the canvas.
    pub(super) fn draw_spot(&mut self, spot_id: SpotId) {
        let spot = &self.spots[spot_id];

        // Fast path for dark spots
        if spot.illumination <= 0.0 || spot.intensity <= 0.0 {
            return;
        }

        let bbox = BoundingBox::new(spot, self.width, self.height);

        // Check is the spot is clipped out of the canvas.
        if bbox.is_empty() {
            return;
        }

        for i in bbox.y0..bbox.y1 {
            let loff = (i * self.width) as usize;

            for j in bbox.x0..bbox.x1 {
                let poff = loff + j as usize;

                let pixval = self.eval_spot_pixel(spot, j, i);

                // Compose light spot patterns using linear intesity addition
                // with numeric saturation instead of wrapping overflow.
                self.pixbuf[poff] = self.pixbuf[poff].saturating_add(pixval);
            }
        }
    }

    /// Evaluates the spot pixel intensity as a function of the radius vector
    /// drawn from the spot center.
    ///
    /// This version calculates a unit Airy disk pattern deformed
    /// by the `SpotShape` transformation matrix.
    fn eval_spot_pixel(&self, spot: &SpotRec, x: u32, y: u32) -> Pixel {
        // Image pixel intensity range
        // FIXME: Do we need to support 10-bit and 12-bit images here?
        let value_scale = Pixel::MAX as f32;

        // The effective spot position coordinates
        let (posx, posy) = spot.position();

        // Current pixel radius vector
        let rvec = (((x as f32) - posx), ((y as f32) - posy));

        // Transformed radius vector components
        let (tx, ty) = spot.shape_inv.apply(rvec);

        // Transformed radial distance
        let rdist = tx.hypot(ty);

        // Perform pre-computed spot pattern LUT lookup for each pixel:

        // Calculate the LUT index with rounding to the nearest integer.
        let lut_index = (rdist * self.pattern_scale + 0.5) as usize;
        // Transparently zero-extend the pattern function LUT to infinity.
        let pattern_val = self.pattern_lut.get(lut_index).copied().unwrap_or(0.0);

        // Calculate the effective peak intensity.
        let intensity = spot.intensity * spot.illumination * self.brightness;

        // Calculate the final pixel value
        (value_scale * intensity * pattern_val) as Pixel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_radius() {
        let shape = SpotShape::default();

        const RE: f32 = 1.8309;
        let (rx, ry) = shape.effective_radius_xy();

        assert!((rx - RE).abs() < 1e-4, "rx = {}, RE = {}", rx, RE);
        assert!((ry - RE).abs() < 1e-4, "ry = {}, RE = {}", ry, RE);

        let shape = SpotShape {
            xx: 3.0,
            xy: -1.5,
            yx: 2.5,
            yy: 5.0,
        };

        const RX: f32 = 6.1411;
        const RY: f32 = 10.2352;
        let (rx, ry) = shape.effective_radius_xy();

        assert!((rx - RX).abs() < 1e-4, "rx = {}, RX = {}", rx, RX);
        assert!((ry - RY).abs() < 1e-4, "ry = {}, RY = {}", ry, RY);
    }

    #[test]
    fn calc_bbox() {
        let shape = SpotShape::default();
        let shape_inv = shape.invert();
        let position = (7.5, 9.2);
        let offset = (0.0, 0.0);
        let width = 16;
        let height = 16;

        let intensity = 1.0;
        let illumination = 1.0;

        let mut spot = SpotRec {
            position,
            offset,
            shape,
            intensity,
            illumination,
            shape_inv,
        };

        let bbox = BoundingBox::new(&spot, width, height);
        assert!(!bbox.is_empty());
        assert_eq!(bbox.x0, 5);
        assert_eq!(bbox.x1, 10);
        assert_eq!(bbox.y0, 7);
        assert_eq!(bbox.y1, 12);

        spot.position = (10.5, 13.3);

        let bbox = BoundingBox::new(&spot, width, height);
        assert!(!bbox.is_empty());
        assert_eq!(bbox.x0, 8);
        assert_eq!(bbox.x1, 13);
        assert_eq!(bbox.y0, 11);
        assert_eq!(bbox.y1, 16);

        spot.position = (-5.5, 20.3);

        let bbox = BoundingBox::new(&spot, width, height);
        assert!(bbox.is_empty());

        spot.position = (-1.0, 15.5);

        let bbox = BoundingBox::new(&spot, width, height);
        assert!(!bbox.is_empty());
        assert_eq!(bbox.x0, 0);
        assert_eq!(bbox.x1, 1);
        assert_eq!(bbox.y0, 13);
        assert_eq!(bbox.y1, 16);
    }

    #[test]
    fn calc_bbox_rect() {
        let shape = SpotShape {
            xx: 3.0,
            xy: -1.5,
            yx: 2.5,
            yy: 5.0,
        };
        let shape_inv = shape.invert();

        let position = (7.5, 9.2);
        let offset = (0.0, 0.0);
        let width = 32;
        let height = 32;

        let intensity = 1.0;
        let illumination = 1.0;

        let mut spot = SpotRec {
            position,
            offset,
            shape,
            intensity,
            illumination,
            shape_inv,
        };

        let bbox = BoundingBox::new(&spot, width, height);
        assert!(!bbox.is_empty());
        assert_eq!(bbox.x0, 1);
        assert_eq!(bbox.x1, 14);
        assert_eq!(bbox.y0, 0);
        assert_eq!(bbox.y1, 20);

        spot.position = (10.5, 13.3);

        let bbox = BoundingBox::new(&spot, width, height);
        assert!(!bbox.is_empty());
        assert_eq!(bbox.x0, 4);
        assert_eq!(bbox.x1, 17);
        assert_eq!(bbox.y0, 3);
        assert_eq!(bbox.y1, 24);

        spot.position = (-15.5, 20.3);

        let bbox = BoundingBox::new(&spot, width, height);
        assert!(bbox.is_empty());

        spot.position = (-5.0, 15.5);

        let bbox = BoundingBox::new(&spot, width, height);
        assert!(!bbox.is_empty());
        assert_eq!(bbox.x0, 0);
        assert_eq!(bbox.x1, 2);
        assert_eq!(bbox.y0, 5);
        assert_eq!(bbox.y1, 26);
    }

    #[test]
    fn draw_spot() {
        let shape = SpotShape::default();
        let mut c = Canvas::new(8, 8);

        let spot1 = c.add_spot((1.1, 4.3), shape, 0.3);
        let spot2 = c.add_spot((4.6, 7.2), shape, 0.4);
        let spot3 = c.add_spot((6.8, 2.6), shape, 0.4);
        let spot4 = c.add_spot((5.1, 4.6), shape, 0.2);

        c.draw_spot(spot1);
        assert_eq!(c.pixbuf[8 * 4 + 1], 13509);

        c.draw_spot(spot2);
        assert_eq!(c.pixbuf[8 * 7 + 5], 12122);

        c.draw_spot(spot3);
        assert_eq!(c.pixbuf[8 * 3 + 7], 12122);

        c.draw_spot(spot4);
        assert_eq!(c.pixbuf[8 * 5 + 5], 6879);
    }
}
