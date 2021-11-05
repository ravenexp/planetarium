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

use super::{Canvas, EncoderError};

impl Canvas {
    /// Exports the canvas contents in the 8-bit gamma-compressed PNG image format.
    pub(super) fn export_png8bpp(&self) -> Result<Vec<u8>, EncoderError> {
        Err(EncoderError::NotImplemented)
    }

    /// Exports the canvas contents in the 16-bit linear light PNG image format.
    pub(super) fn export_png16bpp(&self) -> Result<Vec<u8>, EncoderError> {
        // TODO: Implement PNG encoder here!
        Ok(Vec::new())
    }
}
