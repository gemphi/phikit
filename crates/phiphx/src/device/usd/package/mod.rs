//! USDZ archive reader — extracts the first `.usda` or `.usdc` layer from a
//! zip-based `.usdz` package and delegates parsing to `UsdaParser`.
//!
//! Mirrors the USDZ ingest path of Apple's USDKit framework.

use std::io::{Cursor, Read};

use super::error::UsdaError;
use super::read::UsdaParser;
use super::stage::UsdStage;

mod mesh;

/// Reads a `.usdz` archive (zip container) and produces a `UsdStage` from the
/// first USD layer found inside.
pub struct UsdzReader;

impl UsdzReader {
    /// Open a byte buffer as a zip archive, locate the first `.usda` or `.usdc`
    /// entry, and parse it into a `UsdStage`.
    ///
    /// Returns `UsdaError::InvalidArchive` if the bytes are not a valid zip.
    /// Returns `UsdaError::NoLayerFound` if no `.usda` or `.usdc` entry exists.
    pub fn read(bytes: &[u8]) -> Result<UsdStage, UsdaError> {
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| UsdaError::InvalidArchive(e.to_string()))?;

        // Locate the first .usda or .usdc entry.
        let entry_index = (0..archive.len()).find(|&i| {
            if let Ok(entry) = archive.by_index(i) {
                let name = entry.name().to_lowercase();
                name.ends_with(".usda") || name.ends_with(".usdc")
            } else {
                false
            }
        });

        let index = entry_index.ok_or(UsdaError::NoLayerFound)?;

        let mut entry = archive
            .by_index(index)
            .map_err(|e| UsdaError::InvalidArchive(e.to_string()))?;

        let mut contents = String::new();
        entry
            .read_to_string(&mut contents)
            .map_err(|e| UsdaError::InvalidArchive(e.to_string()))?;

        UsdaParser::parse(&contents)
    }
}
