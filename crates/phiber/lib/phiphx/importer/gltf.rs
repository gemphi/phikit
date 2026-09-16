//! `.gltf` / `.glb` format importer
//!
//! Converts glTF 2.0 scenes to point clouds and meshes.

use std::path::Path;

mod full;
mod importer;
mod mesh;
mod scene;
mod transform;

pub use importer::GltfImporter;
pub use scene::GltfScene;

impl GltfImporter {
    /// Unified import entry point — called by [`AssetImporter::import`].
    pub fn import(
        path: &Path,
        progress: &dyn Fn(f32, &str),
    ) -> Result<crate::phiphx::AssetImportResult, String> {
        progress(0.05, "Reading glTF file...");
        let path_str = path.to_string_lossy().to_string();

        let meshes = Self::import_meshes(&path_str).map_err(|e| format!("{e}"))?;

        let obj_count = meshes.len();
        let vert_total: usize = meshes.iter().map(|m| m.positions.len()).sum();
        let obj_names: Vec<String> = meshes.iter().map(|m| m.name.clone()).collect();
        progress(
            0.70,
            &format!("Parsed {obj_count} meshes / {vert_total} verts"),
        );

        let objects: Vec<crate::phiphx::ImportedObject> = meshes
            .iter()
            .map(|m| crate::phiphx::ImportedObject {
                name: m.name.clone(),
                object_type: "MESH".to_string(),
                location: [0.0; 3],
                scale: [1.0; 3],
                vertex_count: m.positions.len(),
                face_count: m.indices.len(),
                material_names: Vec::new(),
                base_color: [0.8, 0.8, 0.8, 1.0],
                roughness: 0.5,
                metallic: 0.0,
                texture_paths: Vec::new(),
                is_instance: false,
            })
            .collect();

        Ok(crate::phiphx::AssetImportResult {
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

#[cfg(test)]
mod tests;
