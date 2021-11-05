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

use super::{Canvas, EncoderError, Pixel};

/// Initial encoded PNG buffer capacity
const PNG_BUF_CAPACITY: usize = 0x10000;

/// Helper function to work around several `png` crate API warts.
///
/// It is essential that `png::Writer` is moved into this function
/// and dropped here!
fn png_write_16bpp<W: Write>(mut writer: Writer<W>, pixels: &[Pixel]) {
    // FIXME: Do we need error handling here?
    let mut stream = writer.stream_writer().unwrap();

    // Convert pixels to 16-bit Big Endian sample data as required
    // by the PNG format specification.
    for p in pixels {
        stream.write_all(&p.to_be_bytes()).unwrap();
    }
}

impl Canvas {
    /// Exports the canvas contents in the 8-bit gamma-compressed PNG image format.
    pub(super) fn export_png8bpp(&self) -> Result<Vec<u8>, EncoderError> {
        Err(EncoderError::NotImplemented)
    }

    /// Exports the canvas contents in the 16-bit linear light PNG image format.
    pub(super) fn export_png16bpp(&self) -> Result<Vec<u8>, EncoderError> {
        // Memory buffer to encode the PNG data to
        let mut pngbuf: Vec<u8> = Vec::with_capacity(PNG_BUF_CAPACITY);

        // Turn `&mut Vec<u8>` into something that implements `std::io::Write`.
        let cursor = Cursor::new(&mut pngbuf);

        let mut encoder = Encoder::new(cursor, self.width, self.height);
        encoder.set_color(ColorType::Grayscale);
        encoder.set_depth(BitDepth::Sixteen);
        encoder.set_source_gamma(ScaledFloat::new(1.0));

        // FIXME: Do we need error handling here?
        let writer = encoder.write_header().unwrap();

        // Do not attempt to inline this!
        png_write_16bpp(writer, self.pixels());

        Ok(pngbuf)
    }
}

#[cfg(test)]
mod tests {
    // use std::fs::write;

    use crate::{ImageFormat, SpotShape};

    use super::*;

    #[test]
    fn export_png16bpp() {
        let w = 256;
        let h = 256;

        let mut c = Canvas::new(w, h);
        c.set_background(0xAA00);
        c.clear();

        let img = c.export_image(ImageFormat::PngLinear16Bpp).unwrap();
        assert_eq!(img.len(), 679);

        // write("test16bpp_1.png", img).unwrap();

        let shape = SpotShape::default().scale(4.5);

        c.add_spot((100.6, 150.2), shape, 0.9);
        c.add_spot((103.8, 146.5), shape, 0.5);

        c.set_background(1000);
        c.draw();

        let img = c.export_image(ImageFormat::PngLinear16Bpp).unwrap();
        assert_eq!(img.len(), 1471);

        // write("test16bpp_2.png", img).unwrap();
    }
}
