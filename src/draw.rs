//! Planetarium
//! ===========
//!
//! Private light spot image drawing routines
//! -----------------------------------------
//!
//! Contains private types and implementations of private methods
//! for the existing public types.

use super::{Canvas, Pixel, SpotId, SpotRec, SpotShape};

impl SpotShape {
    /// Fudge factor for the effective spot radius estimation
    const EFFECTIVE_RADIUS_FACTOR: f32 = 3.0; // TODO: re-evaluate for Airy disks

    /// Calculates the effective radius of the spot image.
    fn effective_radius(&self) -> f32 {
        // FIXME:
        //   Come up with a better effective radius estimation.
        //   Linear operator norm looks good, but is difficult to calculate.

        // Re = F*max(sqrt(a11^2 + a22^2), sqrt(a12^2 + a21^2))
        Self::EFFECTIVE_RADIUS_FACTOR * self.xx.hypot(self.yy).max(self.xy.hypot(self.yx))
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
        let re = spot.shape.effective_radius();
        let (px, py) = spot.position;
        let (w, h) = (width as i32, height as i32);

        let x0 = ((px - re).floor() as i32).max(0).min(w) as u32;
        let y0 = ((py - re).floor() as i32).max(0).min(h) as u32;
        let x1 = ((px + re).ceil() as i32).max(0).min(w) as u32;
        let y1 = ((py + re).ceil() as i32).max(0).min(h) as u32;

        BoundingBox { x0, y0, x1, y1 }
    }

    /// Checks if the bounding box is contains no pixels.
    fn is_empty(&self) -> bool {
        self.x0 == self.x1 || self.y0 == self.y1
    }
}

impl Canvas {
    /// Draws a single light spot image on the canvas.
    pub(super) fn draw_spot(&mut self, spot_id: SpotId) {
        let spot = &self.spots[spot_id];

        // Fast path for dark spots
        if spot.intensity <= 0.0 {
            return;
        }

        let bbox = BoundingBox::new(spot, self.width, self.height);

        // Check is the spot is clipped out of the canvas.
        if bbox.is_empty() {
            return;
        }

        // TODO: Pixel value must depend on the coordinates
        //       for more complex shapes (e.g. Airy disk).
        let scale = Pixel::MAX as f32;
        let pixel = (scale * spot.intensity * self.brightness) as Pixel;

        for i in bbox.y0..bbox.y1 {
            let loff = (i * self.width) as usize;

            for j in bbox.x0..bbox.x1 {
                let coff = j as usize;

                // TODO: Evaluate pixel intensity as a function of distance
                //       from the spot center.
                self.pixbuf[loff + coff] = pixel;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_radius() {
        let shape = SpotShape::default();

        const RE: f32 = 4.2426;
        let re = shape.effective_radius();

        assert!((re - RE).abs() < 1e-3, "re = {}, RE = {}", re, RE);
    }

    #[test]
    fn calc_bbox() {
        let shape = SpotShape::default();
        let position = (7.5, 9.2);
        let width = 16;
        let height = 16;

        let intensity = 1.0;

        let mut spot = SpotRec {
            position,
            intensity,
            shape,
        };

        let bbox = BoundingBox::new(&spot, width, height);
        assert!(!bbox.is_empty());
        assert_eq!(bbox.x0, 3);
        assert_eq!(bbox.x1, 12);
        assert_eq!(bbox.y0, 4);
        assert_eq!(bbox.y1, 14);

        spot.position = (10.5, 13.3);

        let bbox = BoundingBox::new(&spot, width, height);
        assert!(!bbox.is_empty());
        assert_eq!(bbox.x0, 6);
        assert_eq!(bbox.x1, 15);
        assert_eq!(bbox.y0, 9);
        assert_eq!(bbox.y1, 16);

        spot.position = (-5.5, 20.3);

        let bbox = BoundingBox::new(&spot, width, height);
        assert!(bbox.is_empty());

        spot.position = (-2.5, 15.3);

        let bbox = BoundingBox::new(&spot, width, height);
        assert!(!bbox.is_empty());
        assert_eq!(bbox.x0, 0);
        assert_eq!(bbox.x1, 2);
        assert_eq!(bbox.y0, 11);
        assert_eq!(bbox.y1, 16);
    }
}
