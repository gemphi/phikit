//! Error types for the USDKit USDA parser, printer, and USDZ reader.
//!
//! Mirrors error conditions encountered when loading or validating USD scene
//! data — missing magic headers, malformed attribute values, invalid archives,
//! and absent layers.

/// Errors produced by the USDA parser, USDZ reader, and related utilities.
#[derive(Debug, thiserror::Error)]
pub enum UsdaError {
    /// The input does not begin with the required `#usda 1.0` magic header.
    #[error("missing #usda magic header")]
    MissingHeader,

    /// A value could not be parsed for a specific attribute on a prim.
    #[error("parse error at prim {path}, attribute {attr}: {msg}")]
    ParseError {
        /// The absolute prim path where the error occurred.
        path: String,
        /// The attribute name that failed to parse.
        attr: String,
        /// A human-readable description of the parse failure.
        msg: String,
    },

    /// The byte buffer is not a valid zip archive.
    #[error("invalid zip archive: {0}")]
    InvalidArchive(String),

    /// No `.usda` or `.usdc` layer was found inside the archive.
    #[error("no USD layer found in archive")]
    NoLayerFound,
}
