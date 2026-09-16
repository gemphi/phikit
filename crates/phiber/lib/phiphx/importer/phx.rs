//! PHX format importer.

use crate::phiphx::AssetImportResult;
use crate::phx::exporter::scene as exporter_scene;
use std::path::Path;

/// Importer for PHX binary scenes.
pub struct PhxImporter;

impl PhxImporter {
    /// Import a PHX scene from a file path.
    pub fn import(path: &Path, progress: &dyn Fn(f32, &str)) -> Result<AssetImportResult, String> {
        progress(0.05, "Reading PHX file...");
        let path_str = path.to_string_lossy().to_string();

        let data = std::fs::read(path).map_err(|e| format!("{e}"))?;
        let scene = exporter_scene::import_scene_from_bytes(&data).map_err(|e| format!("{e:?}"))?;

        let obj_count = scene.meshes.len();
        let vert_total: usize = scene.meshes.iter().map(|m| m.vertices.len()).sum();
        let obj_names: Vec<String> = scene.meshes.iter().map(|m| m.name.clone()).collect();
        progress(
            0.70,
            &format!("Parsed {obj_count} meshes / {vert_total} verts"),
        );

        let objects: Vec<crate::phiphx::ImportedObject> = scene
            .meshes
            .iter()
            .map(|m| crate::phiphx::ImportedObject {
                name: m.name.clone(),
                object_type: "MESH".to_string(),
                location: [
                    m.matrix_world[3][0],
                    m.matrix_world[3][1],
                    m.matrix_world[3][2],
                ],
                scale: [1.0; 3],
                vertex_count: m.vertices.len(),
                face_count: m.indices.len(),
                material_names: m.materials.clone(),
                base_color: [0.8, 0.8, 0.8, 1.0],
                roughness: 0.5,
                metallic: 0.0,
                texture_paths: Vec::new(),
                is_instance: false,
            })
            .collect();

        Ok(AssetImportResult {
            mesh_path: path_str,
            vert_total,
            obj_count,
            obj_names,
            bounds: None,
            objects,
            materials: Vec::new(),
            collections: Vec::new(),
        })
    }
}
