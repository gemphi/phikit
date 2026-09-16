//! Unified asset importer — the **single entry point** for all format imports.

use super::FileFormat;
use super::blend::BlendImporter;
#[cfg(feature = "gltf-import")]
use super::gltf::GltfImporter;
use super::phx::PhxImporter;
use super::ply::PlyImporter;
use super::usd::UsdImporter;
use glam::Vec3;
use std::path::Path;

/// Format-agnostic result of importing any supported asset.
#[derive(Debug, Clone)]
pub struct AssetImportResult {
    /// The mesh path reference the GPU loader should use to load this asset.
    pub mesh_path: String,
    /// Number of vertices across all objects.
    pub vert_total: usize,
    /// Number of objects / meshes.
    pub obj_count: usize,
    /// Object names for the hierarchy / outliner panel.
    pub obj_names: Vec<String>,
    /// World-space bounding sphere (center, radius) for camera auto-framing.
    pub bounds: Option<(Vec3, f32)>,
    /// Detailed object info (transform, vertex/face counts, material names).
    pub objects: Vec<ImportedObject>,
    /// Materials extracted from the file (deduplicated by name).
    pub materials: Vec<ImportedMaterial>,
    /// Collection / scene hierarchy (name → object indices).
    pub collections: Vec<ImportedCollection>,
}

/// Lightweight object info for UI display (no heavy vertex buffers).
#[derive(Debug, Clone)]
pub struct ImportedObject {
    pub name: String,
    pub object_type: String,
    pub location: [f32; 3],
    pub scale: [f32; 3],
    pub vertex_count: usize,
    pub face_count: usize,
    pub material_names: Vec<String>,
    pub base_color: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
    pub texture_paths: Vec<String>,
    pub is_instance: bool,
}

/// Material info for the properties panel.
#[derive(Debug, Clone)]
pub struct ImportedMaterial {
    pub name: String,
    pub base_color: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
    pub base_color_texture: Option<String>,
}

/// Collection / group node in the scene hierarchy.
#[derive(Debug, Clone)]
pub struct ImportedCollection {
    pub name: String,
    pub object_indices: Vec<usize>,
    pub children: Vec<ImportedCollection>,
}

/// Structure managing the unified asset import system.
pub struct AssetImporter;

impl AssetImporter {
    /// Import any supported asset file, reporting progress to the callback.
    ///
    /// This is the **only** function an app needs to call. The format is detected
    /// from the file extension and the right importer is invoked.
    pub fn import(path: &Path, progress: &dyn Fn(f32, &str)) -> Result<AssetImportResult, String> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let fmt = FileFormat::from_extension(&ext)
            .ok_or_else(|| format!("Unsupported file format: .{ext}"))?;

        match fmt {
            FileFormat::Blend => BlendImporter::import(path, progress),
            #[cfg(feature = "gltf-import")]
            FileFormat::Gltf | FileFormat::Glb => GltfImporter::import(path, progress),
            FileFormat::Usd | FileFormat::Usda | FileFormat::Usdc => {
                UsdImporter::import(path, progress)
            }
            #[cfg(feature = "usdz-import")]
            FileFormat::Usdz => UsdImporter::import(path, progress),
            FileFormat::Ply => PlyImporter::import(path, progress),
            FileFormat::Phx => PhxImporter::import(path, progress),
            _ => Err(format!("Unsupported file format: .{ext}")),
        }
    }
}
