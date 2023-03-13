//! Planetarium
//! ===========
//!
//! Private RAW image export routines
//! ---------------------------------
//!
//! Contains implementations of private methods
//! for the existing public types.

use crate::{Canvas, EncoderError, Window};

#[allow(clippy::unnecessary_wraps)]
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

    /// Exports the subsampled canvas contents in the 8-bit gamma-compressed
    /// RAW image format.
    pub(super) fn export_sub_raw8bpp(&self, factors: (u32, u32)) -> Result<Vec<u8>, EncoderError> {
        // Subsampled image size in pixels
        let pixlen = self.pixbuf.len() / (factors.0 * factors.1) as usize;

        // Memory buffer to encode the RAW pixel data to
        let mut rawbuf: Vec<u8> = Vec::with_capacity(pixlen);

        for i in 0..(self.height / factors.1) {
            let loffset = (i * factors.1 * self.width) as usize;

            for j in 0..(self.width / factors.0) {
                let offset = loffset + (j * factors.0) as usize;
                let xval = self.gamma_curve.transform(self.pixbuf[offset]);
                rawbuf.push(xval);
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
    use crate::{ImageFormat, SpotShape};

    use super::*;

    /// Creates a 256x256 canvas image for all tests.
    fn mkimage() -> Canvas {
        let mut c = Canvas::new(256, 256);
        c.set_background(1000);

        let shape = SpotShape::default().scale(4.5);
        let shape2 = shape.stretch(1.7, 0.7).rotate(45.0);

        c.add_spot((100.6, 150.2), shape, 0.9);
        c.add_spot((103.8, 146.5), shape2, 0.5);

        c.draw();
        c
    }

    #[test]
    fn export_raw8bpp() {
        let img = mkimage().export_image(ImageFormat::RawGamma8Bpp).unwrap();
        assert_eq!(img.len(), 256 * 256);
        assert_eq!(img[0], 33);
        assert_eq!(img[150 * 256 + 100], 238);
    }

    #[test]
    fn export_sub_raw8bpp() {
        let img = mkimage()
            .export_subsampled_image((2, 2), ImageFormat::RawGamma8Bpp)
            .unwrap();
        assert_eq!(img.len(), 256 * 256 / 2 / 2);
        assert_eq!(img[0], 33);
        assert_eq!(img[(150 * 128 + 100) / 2], 238);
    }

    #[test]
    fn export_window_raw8bpp() {
        let wnd = Window::new(32, 16).at(90, 140);

        let img = mkimage()
            .export_window_image(wnd, ImageFormat::RawGamma8Bpp)
            .unwrap();
        assert_eq!(img.len(), wnd.len());
        assert_eq!(img[300], 186);
    }

    #[test]
    fn export_raw10bpp() {
        let img = mkimage()
            .export_image(ImageFormat::RawLinear10BppLE)
            .unwrap();
        assert_eq!(img.len(), 256 * 256 * 2);
        assert_eq!(img[0], 0x0F);
        assert_eq!(img[1], 0x00);
        assert_eq!(img[2 * (150 * 256 + 100)], 104);
        assert_eq!(img[2 * (150 * 256 + 100) + 1], 3);
    }

    #[test]
    fn export_sub_raw10bpp() {
        let img = mkimage()
            .export_subsampled_image((2, 2), ImageFormat::RawLinear10BppLE)
            .unwrap();
        assert_eq!(img.len(), 256 * 256 * 2 / 2 / 2);
        assert_eq!(img[0], 0x0F);
        assert_eq!(img[1], 0x00);
        assert_eq!(img[2 * (150 * 128 + 100) / 2], 104);
        assert_eq!(img[2 * (150 * 128 + 100) / 2 + 1], 3);
    }

    #[test]
    fn export_window_raw10bpp() {
        let wnd = Window::new(32, 16).at(90, 140);

        let img = mkimage()
            .export_window_image(wnd, ImageFormat::RawLinear10BppLE)
            .unwrap();
        assert_eq!(img.len(), 2 * wnd.len());
        assert_eq!(img[300 * 2], 243);
        assert_eq!(img[300 * 2 + 1], 1);
    }

    #[test]
    fn export_raw12bpp() {
        let img = mkimage()
            .export_image(ImageFormat::RawLinear12BppLE)
            .unwrap();
        assert_eq!(img.len(), 256 * 256 * 2);
        assert_eq!(img[0], 0x3E);
        assert_eq!(img[1], 0x00);
        assert_eq!(img[2 * (150 * 256 + 100)], 162);
        assert_eq!(img[2 * (150 * 256 + 100) + 1], 13);
    }

    #[test]
    fn export_sub_raw12bpp() {
        let img = mkimage()
            .export_subsampled_image((4, 2), ImageFormat::RawLinear12BppLE)
            .unwrap();
        assert_eq!(img.len(), 256 * 256 * 2 / 4 / 2);
        assert_eq!(img[0], 0x3E);
        assert_eq!(img[1], 0x00);
        assert_eq!(img[2 * (150 / 2 * 64 + 100 / 4)], 162);
        assert_eq!(img[2 * (150 / 2 * 64 + 100 / 4) + 1], 13);
    }
}
