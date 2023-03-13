//! Planetarium
//! ===========
//!
//! Private PNG image export routines
//! ---------------------------------
//!
//! This module is gated by the "png" feature.
//!
//! Contains implementations of optional private methods
//! for the existing public types.

use std::io::{Cursor, Write};

use png::{BitDepth, ColorType, Encoder, ScaledFloat};

use crate::{Canvas, EncoderError, Window};

/// Initial encoded PNG buffer capacity
const PNG_BUF_CAPACITY: usize = 0x10000;

#[allow(clippy::unnecessary_wraps)]
impl Canvas {
    /// Exports the canvas window contents in the 8-bit gamma-compressed PNG image format.
    pub(super) fn export_png8bpp(&self, window: Window) -> Result<Vec<u8>, EncoderError> {
        // Memory buffer to encode the PNG data to
        let mut pngbuf: Vec<u8> = Vec::with_capacity(PNG_BUF_CAPACITY);

        // Turn `&mut Vec<u8>` into something that implements `std::io::Write`.
        let cursor = Cursor::new(&mut pngbuf);

        let mut encoder = Encoder::new(cursor, window.w, window.h);
        encoder.set_color(ColorType::Grayscale);
        encoder.set_depth(BitDepth::Eight);
        // sRGB compression gamma = 1 / 2.2 = 0.45455 (rounded)
        encoder.set_source_gamma(ScaledFloat::from_scaled(45455));

        // FIXME: Do we need error handling here?
        let mut writer = encoder.write_header().unwrap();
        let mut stream = writer.stream_writer().unwrap();

        // The window is bounds checked by the caller.
        let spans = self.window_spans(window).unwrap();

        for span in spans {
            // Convert pixels to 8-bit sRGB grayscale sample data.
            for &p in span {
                let gray8 = self.gamma_curve.transform(p);
                stream.write_all(&[gray8]).unwrap();
            }
        }

        // Both PNG writers must be dropped here to release `pngbuf`.
        stream.finish().unwrap();
        writer.finish().unwrap();

        Ok(pngbuf)
    }

    /// Exports the canvas window contents in the 16-bit linear light PNG image format.
    pub(super) fn export_png16bpp(&self, window: Window) -> Result<Vec<u8>, EncoderError> {
        // Memory buffer to encode the PNG data to
        let mut pngbuf: Vec<u8> = Vec::with_capacity(PNG_BUF_CAPACITY);

        // Turn `&mut Vec<u8>` into something that implements `std::io::Write`.
        let cursor = Cursor::new(&mut pngbuf);

        let mut encoder = Encoder::new(cursor, window.w, window.h);
        encoder.set_color(ColorType::Grayscale);
        encoder.set_depth(BitDepth::Sixteen);
        encoder.set_source_gamma(ScaledFloat::new(1.0));

        // FIXME: Do we need error handling here?
        let mut writer = encoder.write_header().unwrap();
        let mut stream = writer.stream_writer().unwrap();

        // The window is bounds checked by the caller.
        let spans = self.window_spans(window).unwrap();

        for span in spans {
            // Convert pixels to 16-bit Big Endian sample data as required
            // by the PNG format specification.
            for p in span {
                stream.write_all(&p.to_be_bytes()).unwrap();
            }
        }

        // Both PNG writers must be dropped here to release `pngbuf`.
        stream.finish().unwrap();
        writer.finish().unwrap();

        Ok(pngbuf)
    }

    /// Exports the subsampled canvas contents in the 8-bit gamma-compressed
    /// PNG image format.
    pub(super) fn export_sub_png8bpp(&self, factors: (u32, u32)) -> Result<Vec<u8>, EncoderError> {
        // Memory buffer to encode the PNG data to
        let mut pngbuf: Vec<u8> = Vec::with_capacity(PNG_BUF_CAPACITY);

        // Subsampled image dimensions
        let width = self.width / factors.0;
        let height = self.height / factors.1;

        // Turn `&mut Vec<u8>` into something that implements `std::io::Write`.
        let cursor = Cursor::new(&mut pngbuf);

        let mut encoder = Encoder::new(cursor, width, height);
        encoder.set_color(ColorType::Grayscale);
        encoder.set_depth(BitDepth::Eight);
        // sRGB compression gamma = 1 / 2.2 = 0.45455 (rounded)
        encoder.set_source_gamma(ScaledFloat::from_scaled(45455));

        // FIXME: Do we need error handling here?
        let mut writer = encoder.write_header().unwrap();
        let mut stream = writer.stream_writer().unwrap();

        for i in 0..(self.height / factors.1) {
            let loffset = (i * factors.1 * self.width) as usize;

            for j in 0..(self.width / factors.0) {
                let offset = loffset + (j * factors.0) as usize;
                let gray8 = self.gamma_curve.transform(self.pixbuf[offset]);
                stream.write_all(&[gray8]).unwrap();
            }
        }

        // Both PNG writers must be dropped here to release `pngbuf`.
        stream.finish().unwrap();
        writer.finish().unwrap();

        Ok(pngbuf)
    }

    /// Exports the subsampled canvas contents in the 16-bit linear light
    /// PNG image format.
    pub(super) fn export_sub_png16bpp(&self, factors: (u32, u32)) -> Result<Vec<u8>, EncoderError> {
        // Memory buffer to encode the PNG data to
        let mut pngbuf: Vec<u8> = Vec::with_capacity(PNG_BUF_CAPACITY);

        // Subsampled image dimensions
        let width = self.width / factors.0;
        let height = self.height / factors.1;

        // Turn `&mut Vec<u8>` into something that implements `std::io::Write`.
        let cursor = Cursor::new(&mut pngbuf);

        let mut encoder = Encoder::new(cursor, width, height);
        encoder.set_color(ColorType::Grayscale);
        encoder.set_depth(BitDepth::Sixteen);
        encoder.set_source_gamma(ScaledFloat::new(1.0));

        // FIXME: Do we need error handling here?
        let mut writer = encoder.write_header().unwrap();
        let mut stream = writer.stream_writer().unwrap();

        for i in 0..(self.height / factors.1) {
            let loffset = (i * factors.1 * self.width) as usize;

            for j in 0..(self.width / factors.0) {
                let offset = loffset + (j * factors.0) as usize;

                // Convert pixels to 16-bit Big Endian sample data as required
                // by the PNG format specification.
                let bytes = self.pixbuf[offset].to_be_bytes();
                stream.write_all(&bytes).unwrap();
            }
        }

        // Both PNG writers must be dropped here to release `pngbuf`.
        stream.finish().unwrap();
        writer.finish().unwrap();

        Ok(pngbuf)
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
    fn export_png8bpp() {
        let img = mkimage().export_image(ImageFormat::PngGamma8Bpp).unwrap();
        assert_eq!(img.len(), 1258);
    }

    #[test]
    fn export_window_png8bpp() {
        let wnd = Window::new(32, 16).at(90, 140);

        let img = mkimage()
            .export_window_image(wnd, ImageFormat::PngGamma8Bpp)
            .unwrap();
        assert_eq!(img.len(), 423);
    }

    #[test]
    fn export_sub_png8bpp() {
        let img = mkimage()
            .export_subsampled_image((2, 2), ImageFormat::PngGamma8Bpp)
            .unwrap();
        assert_eq!(img.len(), 405);
    }

    #[test]
    fn export_png16bpp() {
        let img = mkimage().export_image(ImageFormat::PngLinear16Bpp).unwrap();
        assert_eq!(img.len(), 2550);
    }

    #[test]
    fn export_window_png16bpp() {
        let wnd = Window::new(32, 16).at(90, 140);

        let img = mkimage()
            .export_window_image(wnd, ImageFormat::PngLinear16Bpp)
            .unwrap();
        assert_eq!(img.len(), 765);
    }

    #[test]
    fn export_sub_png16bpp() {
        let img = mkimage()
            .export_subsampled_image((2, 2), ImageFormat::PngLinear16Bpp)
            .unwrap();
        assert_eq!(img.len(), 720);
    }
}
