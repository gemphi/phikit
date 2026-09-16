//! `UsdImporter` definition and public entry points.

use crate::phx::importer::Importer;
use crate::phx::{Mesh, MeshToCloudConfig};
use crate::phx::{PointCloud, PointCloudError, Result};

/// USD importer
pub struct UsdImporter;

impl UsdImporter {
    /// Check if data is a valid USD file
    pub fn can_import(data: &[u8]) -> bool {
        // USDA starts with #usda
        if data.starts_with(b"#usda") {
            return true;
        }

        // USDC is binary, check for crate magic
        if data.len() >= 8 {
            // "PXR-USDC" magic
            if &data[0..8] == b"PXR-USDC" {
                return true;
            }
        }

        // USDZ is a zip file (PK\x03\x04 or PK\x05\x06)
        if data.len() >= 4 {
            if data[0] == 0x50 && data[1] == 0x4B {
                // PK signature for zip
                if data[2] == 0x03 || data[2] == 0x05 || data[2] == 0x07 {
                    return true;
                }
            }
        }

        false
    }

    /// Import USD from bytes
    pub fn import_bytes(data: &[u8]) -> Result<PointCloud> {
        let config = MeshToCloudConfig::default();
        Self::import_bytes_with_config(data, &config)
    }

    /// Import with custom configuration
    pub fn import_bytes_with_config(data: &[u8], config: &MeshToCloudConfig) -> Result<PointCloud> {
        if data.starts_with(b"#usda") {
            // ASCII format
            let text = std::str::from_utf8(data)
                .map_err(|e| PointCloudError::ImportError(format!("Invalid UTF-8: {}", e)))?;
            Self::import_usda(text, config)
        } else if data.len() >= 8 && &data[0..8] == b"PXR-USDC" {
            // Binary crate format — extract float arrays heuristically
            Self::import_usdc_binary(data, config)
        } else if data.len() >= 4 && data[0] == 0x50 && data[1] == 0x4B {
            // USDZ zip format
            #[cfg(feature = "usdz-import")]
            {
                Self::import_usdz(data, config)
            }
            #[cfg(not(feature = "usdz-import"))]
            {
                Err(PointCloudError::ImportError(
                    "USDZ import requires `usdz-import` feature (zip crate)".to_string(),
                ))
            }
        } else {
            Err(PointCloudError::InvalidFormat(
                "Unknown USD format".to_string(),
            ))
        }
    }

    /// Import from file path
    pub fn import_file(path: &str) -> Result<PointCloud> {
        let config = MeshToCloudConfig::default();
        Self::import_file_with_config(path, &config)
    }

    /// Import from file with custom configuration
    pub fn import_file_with_config(path: &str, config: &MeshToCloudConfig) -> Result<PointCloud> {
        let data = std::fs::read(path)
            .map_err(|e| PointCloudError::ImportError(format!("Failed to read file: {}", e)))?;

        let mut cloud = Self::import_bytes_with_config(&data, config)?;
        cloud.metadata.source_path = Some(path.to_string());
        cloud.metadata.source_format = Some("USD".to_string());

        Ok(cloud)
    }

    /// Import USD as full Mesh (preserves triangles)
    pub fn import_file_as_mesh(path: &str) -> Result<Vec<Mesh>> {
        let data = std::fs::read(path)
            .map_err(|e| PointCloudError::ImportError(format!("Failed to read file: {}", e)))?;

        if data.starts_with(b"#usda") {
            let text = std::str::from_utf8(&data)
                .map_err(|e| PointCloudError::ImportError(format!("Invalid UTF-8: {}", e)))?;
            Self::import_usda_as_mesh(text)
        } else if data.len() >= 4 && data[0] == 0x50 && data[1] == 0x4B {
            // USDZ - extract and parse
            #[cfg(feature = "usdz-import")]
            {
                Self::import_usdz_as_mesh(&data)
            }
            #[cfg(not(feature = "usdz-import"))]
            {
                Err(PointCloudError::ImportError(
                    "USDZ import requires `usdz-import` feature (zip crate)".to_string(),
                ))
            }
        } else if data.len() >= 8 && &data[0..8] == b"PXR-USDC" {
            // Heuristic binary extraction for mesh
            Self::import_usdc_as_mesh(&data)
        } else {
            Err(PointCloudError::ImportError(format!(
                "Unsupported USD format for mesh import: {}. Only ASCII (.usda), Zip (.usdz), and binary (.usdc) heuristic extraction are supported.",
                path
            )))
        }
    }
}

impl Importer for UsdImporter {
    fn can_import(data: &[u8]) -> bool {
        Self::can_import(data)
    }

    fn import_bytes(data: &[u8]) -> Result<PointCloud> {
        Self::import_bytes(data)
    }

    fn import_file(path: &str) -> Result<PointCloud> {
        Self::import_file(path)
    }
}
