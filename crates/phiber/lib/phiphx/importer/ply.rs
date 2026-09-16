//! PLY format importer.

use crate::phiphx::AssetImportResult;
use crate::phx::{PointCloud, PointCloudFormat};
use std::path::Path;

/// Importer for PLY (Stanford Triangle Format) point clouds.
pub struct PlyImporter;

impl PlyImporter {
    /// Import a PLY point cloud from a file path.
    pub fn import(path: &Path, progress: &dyn Fn(f32, &str)) -> Result<AssetImportResult, String> {
        progress(0.05, "Reading PLY file...");
        let path_str = path.to_string_lossy().to_string();

        let text = std::fs::read_to_string(path).map_err(|e| format!("{e}"))?;
        let cloud = PointCloud::import(
            &text,
            &PointCloudFormat::Ply {
                binary: false,
                include_colors: true,
                include_normals: true,
            },
        )
        .map_err(|e| format!("{e}"))?;

        let vert_total = cloud.vertex_count();
        progress(0.70, &format!("Parsed {vert_total} points"));

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("mesh")
            .to_string();
        let objects = vec![crate::phiphx::ImportedObject {
            name: name.clone(),
            object_type: "POINTCLOUD".to_string(),
            location: [0.0; 3],
            scale: [1.0; 3],
            vertex_count: vert_total,
            face_count: 0,
            material_names: Vec::new(),
            base_color: [0.8, 0.8, 0.8, 1.0],
            roughness: 0.5,
            metallic: 0.0,
            texture_paths: Vec::new(),
            is_instance: false,
        }];

        Ok(AssetImportResult {
            mesh_path: path_str,
            vert_total,
            obj_count: 1,
            obj_names: vec![name],
            bounds: None,
            objects,
            materials: Vec::new(),
            collections: Vec::new(),
        })
    }
}
