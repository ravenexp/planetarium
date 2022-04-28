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

use png::{BitDepth, ColorType, Encoder, ScaledFloat, Writer};

use crate::{Canvas, EncoderError, Window, WindowSpans};

use crate::gamma::GammaCurve8;

/// Initial encoded PNG buffer capacity
const PNG_BUF_CAPACITY: usize = 0x10000;

/// Helper function to work around several `png` crate API warts.
///
/// It is essential that `png::Writer` is moved into this function
/// and dropped here!
fn png_write_8bpp<W: Write>(mut writer: Writer<W>, spans: WindowSpans, gamma: &GammaCurve8) {
    // FIXME: Do we need error handling here?
    let mut stream = writer.stream_writer().unwrap();

    for span in spans {
        // Convert pixels to 8-bit sRGB grayscale sample data.
        for &p in span {
            let gray8 = gamma.transform(p);
            stream.write_all(&[gray8]).unwrap();
        }
    }
}

/// Helper function to work around several `png` crate API warts.
///
/// It is essential that `png::Writer` is moved into this function
/// and dropped here!
fn png_write_16bpp<W: Write>(mut writer: Writer<W>, spans: WindowSpans) {
    // FIXME: Do we need error handling here?
    let mut stream = writer.stream_writer().unwrap();

    for span in spans {
        // Convert pixels to 16-bit Big Endian sample data as required
        // by the PNG format specification.
        for p in span {
            stream.write_all(&p.to_be_bytes()).unwrap();
        }
    }
}

/// Helper function (subsampling version) to work around several
/// `png` crate API warts.
///
/// It is essential that `png::Writer` is moved into this function
/// and dropped here!
fn png_write_sub_8bpp<W: Write>(mut writer: Writer<W>, canvas: &Canvas, factors: (u32, u32)) {
    // FIXME: Do we need error handling here?
    let mut stream = writer.stream_writer().unwrap();

    for i in 0..(canvas.height / factors.1) {
        let loffset = (i * factors.1 * canvas.width) as usize;

        for j in 0..(canvas.width / factors.0) {
            let offset = loffset + (j * factors.0) as usize;
            let gray8 = canvas.gamma_curve.transform(canvas.pixbuf[offset]);
            stream.write_all(&[gray8]).unwrap();
        }
    }
}

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
        let writer = encoder.write_header().unwrap();

        // The window is bounds checked by the caller.
        let spans = self.window_spans(window).unwrap();

        // Do not attempt to inline this!
        png_write_8bpp(writer, spans, &self.gamma_curve);

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
        let writer = encoder.write_header().unwrap();

        // The window is bounds checked by the caller.
        let spans = self.window_spans(window).unwrap();

        // Do not attempt to inline this!
        png_write_16bpp(writer, spans);

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
        let writer = encoder.write_header().unwrap();

        // Do not attempt to inline this!
        png_write_sub_8bpp(writer, self, factors);

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
        assert_eq!(img.len(), 912);
    }

    #[test]
    fn export_window_png8bpp() {
        let wnd = Window::new(32, 16).at(90, 140);

        let img = mkimage()
            .export_window_image(wnd, ImageFormat::PngGamma8Bpp)
            .unwrap();
        assert_eq!(img.len(), 413);
    }

    #[test]
    fn export_sub_png8bpp() {
        let img = mkimage()
            .export_subsampled_image((2, 2), ImageFormat::PngGamma8Bpp)
            .unwrap();
        assert_eq!(img.len(), 340);
    }

    #[test]
    fn export_png16bpp() {
        let img = mkimage().export_image(ImageFormat::PngLinear16Bpp).unwrap();
        assert_eq!(img.len(), 1621);
    }

    #[test]
    fn export_window_png16bpp() {
        let wnd = Window::new(32, 16).at(90, 140);

        let img = mkimage()
            .export_window_image(wnd, ImageFormat::PngLinear16Bpp)
            .unwrap();
        assert_eq!(img.len(), 756);
    }
}
