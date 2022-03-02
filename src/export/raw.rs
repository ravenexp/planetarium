//! Planetarium
//! ===========
//!
//! Private RAW image export routines
//! ---------------------------------
//!
//! Contains implementations of private methods
//! for the existing public types.

use crate::{Canvas, EncoderError, Window};

impl Canvas {
    /// Exports the canvas window contents in the 8-bit gamma-compressed RAW image format.
    pub(super) fn export_raw8bpp(&self, window: Window) -> Result<Vec<u8>, EncoderError> {
        // Memory buffer to encode the RAW pixel data to
        let mut rawbuf: Vec<u8> = Vec::with_capacity(window.len());

        // The window is bounds checked by the caller.
        for span in self.window_spans(window).unwrap() {
            rawbuf.extend(span.iter().map(|p| self.gamma_curve.transform(*p)));
        }

        Ok(rawbuf)
    }

    /// Exports the canvas window contents in the `X`-bit linear light grayscale
    /// little-endian RAW image format.
    ///
    /// The const generic `X` must be in the range from 9 to 16.
    pub(super) fn export_raw1xbpp<const X: u16>(
        &self,
        window: Window,
    ) -> Result<Vec<u8>, EncoderError> {
        // Memory buffer to encode the RAW pixel data to
        let mut rawbuf: Vec<u8> = Vec::with_capacity(2 * window.len());

        // The window is bounds checked by the caller.
        for span in self.window_spans(window).unwrap() {
            for p in span {
                let bytes = (p >> (16 - X)).to_le_bytes();
                rawbuf.extend_from_slice(&bytes);
            }
        }

        Ok(rawbuf)
    }

    /// Exports the subsampled canvas contents in the `X`-bit linear light grayscale
    /// little-endian RAW image format.
    ///
    /// The const generic `X` must be in the range from 9 to 16.
    pub(super) fn export_sub_raw1xbpp<const X: u16>(
        &self,
        factors: (u32, u32),
    ) -> Result<Vec<u8>, EncoderError> {
        // Subsampled image size in pixels
        let pixlen = self.pixbuf.len() / (factors.0 * factors.1) as usize;

        // Memory buffer to encode the RAW pixel data to
        let mut rawbuf: Vec<u8> = Vec::with_capacity(2 * pixlen);

        for i in 0..(self.height / factors.1) {
            let loffset = (i * factors.1 * self.width) as usize;

            for j in 0..(self.width / factors.0) {
                let offset = loffset + (j * factors.0) as usize;
                let bytes = (self.pixbuf[offset] >> (16 - X)).to_le_bytes();
                rawbuf.extend_from_slice(&bytes);
            }
        }

        Ok(rawbuf)
    }
}

#[cfg(test)]
mod tests {
    // use std::fs::write;

    use crate::{ImageFormat, SpotShape};

    use super::*;

    #[test]
    fn export_raw8bpp() {
        let w = 256;
        let h = 256;

        let mut c = Canvas::new(w, h);
        c.set_background(0xAA00);
        c.clear();

        let img = c.export_image(ImageFormat::RawGamma8Bpp).unwrap();
        assert_eq!(img.len(), 65536);

        // write("test8bpp_1.raw", img).unwrap();

        let shape = SpotShape::default().scale(4.5);

        c.add_spot((100.6, 150.2), shape, 0.9);
        c.add_spot((103.8, 146.5), shape, 0.5);

        c.set_background(1000);
        c.draw();

        let img = c.export_image(ImageFormat::RawGamma8Bpp).unwrap();
        assert_eq!(img.len(), 65536);

        // write("test8bpp_2.raw", img).unwrap();
    }

    #[test]
    fn export_window_raw8bpp() {
        let w = 256;
        let h = 256;

        let mut c = Canvas::new(w, h);

        let shape = SpotShape::default().scale(4.5);

        c.add_spot((100.6, 150.2), shape, 0.9);
        c.add_spot((103.8, 146.5), shape, 0.5);

        c.set_background(1000);
        c.draw();

        let wnd = Window::new(32, 16).at(90, 140);

        let img = c
            .export_window_image(wnd, ImageFormat::RawGamma8Bpp)
            .unwrap();
        assert_eq!(img.len(), wnd.len());
        assert_eq!(img[300], 196);

        // write("test8bpp_window.raw", img).unwrap();
    }

    #[test]
    fn export_raw10bpp() {
        let w = 256;
        let h = 256;

        let mut c = Canvas::new(w, h);
        c.set_background(0xAA00);
        c.clear();

        let img = c.export_image(ImageFormat::RawLinear10BppLE).unwrap();
        assert_eq!(img.len(), 131072);
        assert_eq!(img[0], 0xA8);
        assert_eq!(img[1], 0x02);

        // write("test10bpp_1.raw", img).unwrap();

        let shape = SpotShape::default().scale(4.5);

        c.add_spot((100.6, 150.2), shape, 0.9);
        c.add_spot((103.8, 146.5), shape, 0.5);

        c.set_background(1000);
        c.draw();

        let img = c.export_image(ImageFormat::RawLinear10BppLE).unwrap();
        assert_eq!(img.len(), 131072);
        assert_eq!(img[0], 0x0F);
        assert_eq!(img[1], 0x00);

        // write("test10bpp_2.raw", img).unwrap();
    }

    #[test]
    fn export_sub_raw10bpp() {
        let w = 256;
        let h = 256;

        let mut c = Canvas::new(w, h);
        c.set_background(0xAA00);
        c.clear();

        let img = c
            .export_subsampled_image((2, 2), ImageFormat::RawLinear10BppLE)
            .unwrap();
        assert_eq!(img.len(), 32768);
        assert_eq!(img[0], 0xA8);
        assert_eq!(img[1], 0x02);

        // write("test_sub_10bpp_1.raw", img).unwrap();

        let shape = SpotShape::default().scale(4.5);

        c.add_spot((100.6, 150.2), shape, 0.9);
        c.add_spot((103.8, 146.5), shape, 0.5);

        c.set_background(1000);
        c.draw();

        let img = c
            .export_subsampled_image((2, 2), ImageFormat::RawLinear10BppLE)
            .unwrap();
        assert_eq!(img.len(), 32768);
        assert_eq!(img[0], 0x0F);
        assert_eq!(img[1], 0x00);

        // write("test_sub_10bpp_2.raw", img).unwrap();
    }

    #[test]
    fn export_window_raw10bpp() {
        let w = 256;
        let h = 256;

        let mut c = Canvas::new(w, h);

        let shape = SpotShape::default().scale(4.5);

        c.add_spot((100.6, 150.2), shape, 0.9);
        c.add_spot((103.8, 146.5), shape, 0.5);

        c.set_background(1000);
        c.draw();

        let wnd = Window::new(32, 16).at(90, 140);

        let img = c
            .export_window_image(wnd, ImageFormat::RawLinear10BppLE)
            .unwrap();
        assert_eq!(img.len(), 2 * wnd.len());

        // write("test10bpp_window.raw", img).unwrap();
    }

    #[test]
    fn export_raw12bpp() {
        let w = 256;
        let h = 256;

        let mut c = Canvas::new(w, h);
        c.set_background(0xAA00);
        c.clear();

        let img = c.export_image(ImageFormat::RawLinear12BppLE).unwrap();
        assert_eq!(img.len(), 131072);
        assert_eq!(img[0], 0xA0);
        assert_eq!(img[1], 0x0A);

        // write("test12bpp_1.raw", img).unwrap();

        let shape = SpotShape::default().scale(4.5);

        c.add_spot((100.6, 150.2), shape, 0.9);
        c.add_spot((103.8, 146.5), shape, 0.5);

        c.set_background(1000);
        c.draw();

        let img = c.export_image(ImageFormat::RawLinear12BppLE).unwrap();
        assert_eq!(img.len(), 131072);
        assert_eq!(img[0], 0x3E);
        assert_eq!(img[1], 0x00);

        // write("test12bpp_2.raw", img).unwrap();
    }

    #[test]
    fn export_sub_raw12bpp() {
        let w = 256;
        let h = 256;

        let mut c = Canvas::new(w, h);
        c.set_background(0xAA00);
        c.clear();

        let img = c
            .export_subsampled_image((4, 4), ImageFormat::RawLinear12BppLE)
            .unwrap();
        assert_eq!(img.len(), 8192);
        assert_eq!(img[0], 0xA0);
        assert_eq!(img[1], 0x0A);

        // write("test_sub_12bpp_1.raw", img).unwrap();

        let shape = SpotShape::default().scale(4.5);

        c.add_spot((100.6, 150.2), shape, 0.9);
        c.add_spot((103.8, 146.5), shape, 0.5);

        c.set_background(1000);
        c.draw();

        let img = c
            .export_subsampled_image((4, 4), ImageFormat::RawLinear12BppLE)
            .unwrap();
        assert_eq!(img.len(), 8192);
        assert_eq!(img[0], 0x3E);
        assert_eq!(img[1], 0x00);

        // write("test_sub_12bpp_2.raw", img).unwrap();
    }
}
