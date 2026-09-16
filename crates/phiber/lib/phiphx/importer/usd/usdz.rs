//! USDZ (zip container) handling.

use super::UsdImporter;
use crate::phx::{Mesh, MeshToCloudConfig};
use crate::phx::{PointCloud, PointCloudError, Result};
use std::io::Read;

impl UsdImporter {
    /// Import USDZ (zipped USD) format
    pub(crate) fn import_usdz(data: &[u8], config: &MeshToCloudConfig) -> Result<PointCloud> {
        let cursor = std::io::Cursor::new(data);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| PointCloudError::ImportError(format!("Invalid USDZ zip: {}", e)))?;

        // Prefer USDA (text), then USD (could be USDA/USDC), then USDC.
        let mut candidates: Vec<(u8, usize)> = Vec::new();
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                let n = file.name().to_ascii_lowercase();
                if n.ends_with(".usda") || n.ends_with(".usd") || n.ends_with(".usdc") {
                    let rank = if n.ends_with(".usda") {
                        0
                    } else if n.ends_with(".usd") {
                        1
                    } else {
                        2
                    };
                    candidates.push((rank, i));
                }
            }
        }

        // Sort by extension preference.
        candidates.sort_by_key(|(rank, _)| *rank);

        for (_rank, i) in candidates {
            let mut file = archive
                .by_index(i)
                .map_err(|e| PointCloudError::ImportError(format!("USDZ read failed: {}", e)))?;
            let entry_name = file.name().to_string();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).map_err(|e| {
                PointCloudError::ImportError(format!("USDZ entry read failed: {}", e))
            })?;

            // Delegate to the normal USD sniffing logic.
            match Self::import_bytes_with_config(&bytes, config) {
                Ok(mut cloud) => {
                    cloud.metadata.source_path = Some(entry_name);
                    cloud.metadata.source_format = Some("USDZ".to_string());
                    return Ok(cloud);
                }
                Err(e) => {
                    // Continue trying other candidates.
                    let _ = e;
                }
            }
        }

        Err(PointCloudError::ImportError(
            "USDZ archive did not contain a supported USDA/USD payload (binary USDC requires OpenUSD).".to_string()
        ))
    }

    /// Import USDZ as full Meshes.
    ///
    /// USDZ is a zip archive that may contain either USDA (text) or USDC
    /// (binary) payloads. Apple's BOT-anist sample ships USDC inside USDZ,
    /// so we prefer USDA when present but fall back to a heuristic USDC
    /// vertex extraction so callers still get geometry.
    pub(crate) fn import_usdz_as_mesh(data: &[u8]) -> Result<Vec<Mesh>> {
        use zip::ZipArchive;
        let reader = std::io::Cursor::new(data);
        let mut archive = match ZipArchive::new(reader) {
            Ok(a) => a,
            Err(_) => {
                return Err(PointCloudError::ImportError(
                    "Invalid USDZ file".to_string(),
                ));
            }
        };

        // Pass 1: prefer USDA (text) — handles BOTanist's `ground_plane.usda`
        // and any other ASCII payloads.
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| PointCloudError::ImportError(format!("ZIP error: {}", e)))?;
            if file.name().to_ascii_lowercase().ends_with(".usda") {
                let mut contents = String::new();
                std::io::Read::read_to_string(&mut file, &mut contents)
                    .map_err(|e| PointCloudError::ImportError(format!("Read error: {}", e)))?;
                return Self::import_usda_as_mesh(&contents);
            }
        }

        // Pass 2: USDC (binary). Apple's BOT-anist USDZs (head, body,
        // backpack, plant meshes) all carry USDC payloads. We extract
        // vertex positions heuristically — no faces, but the GPU mesh
        // upload path uses sequential triangle topology which is enough
        // to render a recognisable shape.
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| PointCloudError::ImportError(format!("ZIP error: {}", e)))?;
            let name = file.name().to_string();
            let lower = name.to_ascii_lowercase();
            if lower.ends_with(".usdc") || lower.ends_with(".usd") {
                let mut bytes = Vec::new();
                std::io::Read::read_to_end(&mut file, &mut bytes)
                    .map_err(|e| PointCloudError::ImportError(format!("Read error: {}", e)))?;
                if bytes.len() >= 8 && &bytes[0..8] == b"PXR-USDC" {
                    return Self::import_usdc_as_mesh(&bytes);
                }
                // `.usd` may also be USDA text under the hood — try as text.
                if let Ok(text) = std::str::from_utf8(&bytes) {
                    if text.starts_with("#usda") {
                        return Self::import_usda_as_mesh(text);
                    }
                }
            }
        }

        Err(PointCloudError::ImportError(
            "USDZ archive contained no USDA/USDC payload we could parse".to_string(),
        ))
    }
}
