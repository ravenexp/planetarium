//! Planetarium
//! ===========
//!
//! Private light spot image drawing routines
//! -----------------------------------------
//!
//! Contains private types and implementations of private methods
//! for the existing public types.

use super::{Canvas, Pixel, Point, SpotId, SpotShape, Vector};
use crate::pattern::AiryPattern;

impl SpotShape {
    /// Calculates the effective radius of the spot image
    /// projected onto the coordinate axes as XY components.
    #[must_use]
    fn effective_radius_xy(&self) -> (f32, f32) {
        // Rx = F*sqrt(a11^2 + a12^2), Ry = F*sqrt(a22^2 + a21^2))
        (
            AiryPattern::SIZE_FACTOR * self.xx.hypot(self.xy),
            AiryPattern::SIZE_FACTOR * self.yy.hypot(self.yx),
        )
    }

    /// Inverts the shape definition matrix.
    ///
    /// Returns the inverted matrix.
    #[must_use]
    pub(super) fn invert(&self) -> SpotShape {
        let det = self.xx * self.yy - self.xy * self.yx;

        // Bail on (almost) singular matrices in debug builds,
        // fall back to the unit shape in releases.
        if det.abs() < 0.01 {
            debug_assert!(false, "Singular shape matrix: {self:?}");
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
    #[must_use]
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

#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
impl BoundingBox {
    /// Calculates the bounding box for a light spot from its shape and position.
    ///
    /// Clips to box dimensions to the underlying canvas size.
    #[must_use]
    fn new(position: Point, shape: &SpotShape, width: u32, height: u32) -> Self {
        let (rx, ry) = shape.effective_radius_xy();
        let (px, py) = position;
        let (w, h) = (width as i32, height as i32);

        let x0 = ((px - rx).floor() as i32).max(0).min(w) as u32;
        let y0 = ((py - ry).floor() as i32).max(0).min(h) as u32;
        let x1 = ((px + rx).ceil() as i32).max(0).min(w) as u32;
        let y1 = ((py + ry).ceil() as i32).max(0).min(h) as u32;

        BoundingBox { x0, y0, x1, y1 }
    }

    /// Checks if the bounding box is contains no pixels.
    #[must_use]
    fn is_empty(&self) -> bool {
        self.x0 == self.x1 || self.y0 == self.y1
    }
}

#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
impl Canvas {
    /// Draws a single light spot image on the canvas.
    pub(super) fn draw_spot(&mut self, spot_id: SpotId) {
        let position = self.spot_position(spot_id).unwrap();
        let intensity = self.spot_intensity(spot_id).unwrap();

        let shape = self.spots[spot_id].shape;
        let shape_inv = self.spots[spot_id].shape_inv;

        // Fast path for dark spots
        if intensity <= 0.0 {
            return;
        }

        let bbox = BoundingBox::new(position, &shape, self.width, self.height);

        // Check is the spot is clipped out of the canvas.
        if bbox.is_empty() {
            return;
        }

        for i in bbox.y0..bbox.y1 {
            let line_off = (i * self.width) as usize;

            for j in bbox.x0..bbox.x1 {
                let pix_off = line_off + j as usize;

                let pixval = self.eval_spot_pixel(position, &shape_inv, intensity, j, i);

                // Compose light spot patterns using linear intesity addition
                // with numeric saturation instead of wrapping overflow.
                self.pixbuf[pix_off] = self.pixbuf[pix_off].saturating_add(pixval);
            }
        }
    }

    /// Evaluates the spot pixel intensity as a function of the radius vector
    /// drawn from the spot center.
    ///
    /// This version calculates a unit Airy disk pattern deformed
    /// by the `SpotShape` transformation matrix.
    #[must_use]
    fn eval_spot_pixel(
        &self,
        center: Point,
        shape_inv: &SpotShape,
        intensity: f32,
        x: u32,
        y: u32,
    ) -> Pixel {
        // Current pixel radius vector
        let rvec = (((x as f32) - center.0), ((y as f32) - center.1));

        // Transformed radius vector components
        let (tx, ty) = shape_inv.apply(rvec);

        // Transformed radial distance
        let rdist = tx.hypot(ty);

        // Perform pre-computed spot pattern LUT lookup for each pixel.
        let pattern_val = self.pattern.eval(rdist);

        // Calculate the final pixel value
        (intensity * pattern_val * f32::from(Pixel::MAX)) as Pixel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_radius() {
        const RE: f32 = 1.830_9;
        const RX: f32 = 6.141_1;
        const RY: f32 = 10.235_2;

        let shape = SpotShape::default();

        let (rx, ry) = shape.effective_radius_xy();

        assert!((rx - RE).abs() < 1e-4, "rx = {rx}, RE = {RE}");
        assert!((ry - RE).abs() < 1e-4, "ry = {ry}, RE = {RE}");

        let shape = SpotShape {
            xx: 3.0,
            xy: -1.5,
            yx: 2.5,
            yy: 5.0,
        };

        let (rx, ry) = shape.effective_radius_xy();

        assert!((rx - RX).abs() < 1e-4, "rx = {rx}, RX = {RX}");
        assert!((ry - RY).abs() < 1e-4, "ry = {ry}, RY = {RY}");
    }

    #[test]
    fn calc_bbox() {
        let shape = SpotShape::default();
        let mut position = (7.5, 9.2);
        let width = 16;
        let height = 16;

        let bbox = BoundingBox::new(position, &shape, width, height);
        assert!(!bbox.is_empty());
        assert_eq!(bbox.x0, 5);
        assert_eq!(bbox.x1, 10);
        assert_eq!(bbox.y0, 7);
        assert_eq!(bbox.y1, 12);

        position = (10.5, 13.3);

        let bbox = BoundingBox::new(position, &shape, width, height);
        assert!(!bbox.is_empty());
        assert_eq!(bbox.x0, 8);
        assert_eq!(bbox.x1, 13);
        assert_eq!(bbox.y0, 11);
        assert_eq!(bbox.y1, 16);

        position = (-5.5, 20.3);

        let bbox = BoundingBox::new(position, &shape, width, height);
        assert!(bbox.is_empty());

        position = (-1.0, 15.5);

        let bbox = BoundingBox::new(position, &shape, width, height);
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

        let mut position = (7.5, 9.2);
        let width = 32;
        let height = 32;

        let bbox = BoundingBox::new(position, &shape, width, height);
        assert!(!bbox.is_empty());
        assert_eq!(bbox.x0, 1);
        assert_eq!(bbox.x1, 14);
        assert_eq!(bbox.y0, 0);
        assert_eq!(bbox.y1, 20);

        position = (10.5, 13.3);

        let bbox = BoundingBox::new(position, &shape, width, height);
        assert!(!bbox.is_empty());
        assert_eq!(bbox.x0, 4);
        assert_eq!(bbox.x1, 17);
        assert_eq!(bbox.y0, 3);
        assert_eq!(bbox.y1, 24);

        position = (-15.5, 20.3);

        let bbox = BoundingBox::new(position, &shape, width, height);
        assert!(bbox.is_empty());

        position = (-5.0, 15.5);

        let bbox = BoundingBox::new(position, &shape, width, height);
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
        assert_eq!(c.pixbuf[8 * 4 + 1], 13449);

        c.draw_spot(spot2);
        assert_eq!(c.pixbuf[8 * 7 + 5], 11960);

        c.draw_spot(spot3);
        assert_eq!(c.pixbuf[8 * 3 + 7], 11960);

        c.draw_spot(spot4);
        assert_eq!(c.pixbuf[8 * 5 + 5], 6755);
    }
}
