//! Point cloud importer.
//!
//! Handles importing 3D formats (USD, glTF, OBJ) and converting to point clouds.

#[cfg(feature = "gltf-import")]
pub mod gltf;
pub mod phx;
pub mod ply;
pub mod usd;

pub mod asset;
pub mod format;

pub use asset::{
    AssetImportResult, AssetImporter, ImportedCollection, ImportedMaterial, ImportedObject,
};
pub use format::{FileFormat, ObjImporter};
pub use phx::PhxImporter;
pub use ply::PlyImporter;

use crate::{PointCloud, Result};

/// Trait for format-specific importers
pub trait Importer {
    /// Check if this importer can handle the given data
    fn can_import(data: &[u8]) -> bool
    where
        Self: Sized;

    /// Import from bytes
    fn import_bytes(data: &[u8]) -> Result<PointCloud>
    where
        Self: Sized;

    /// Import from file path
    fn import_file(path: &str) -> Result<PointCloud>
    where
        Self: Sized;
}
