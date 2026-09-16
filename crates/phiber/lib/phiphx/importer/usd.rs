//! `.usd` / `.usda` / `.usdc` / `.usdz` format importer
//!
//! Converts Universal Scene Description files to point clouds and meshes.
//! Handles USDA (ASCII), USDC (binary crate), and USDZ (zip container).

use std::path::Path;

mod importer;
mod parse;
mod scene;
mod usda;
mod usda_build;
mod usda_mesh;
mod usda_parse;
mod usdc;
#[cfg(feature = "usdz-import")]
mod usdz;

pub use importer::UsdImporter;
pub use scene::UsdScene;

impl UsdImporter {
    /// Full USD import using external library when available.
    #[cfg(feature = "usdc-parser")]
    pub fn import_with_openusd(
        path: &str,
        config: &crate::phx::MeshToCloudConfig,
    ) -> crate::Result<crate::PointCloud> {
        Self::import_file_with_config(path, config)
    }

    /// Unified import entry point — called by [`AssetImporter::import`].
    pub fn import(
        path: &Path,
        progress: &dyn Fn(f32, &str),
    ) -> Result<crate::phiphx::AssetImportResult, String> {
        progress(0.05, "Reading USD file...");
        let path_str = path.to_string_lossy().to_string();

        let meshes = Self::import_file_as_mesh(&path_str).map_err(|e| format!("{e}"))?;

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
                material_names: vec![format!("{}_mat", m.name)],
                base_color: [0.8, 0.8, 0.8, 1.0],
                roughness: 0.5,
                metallic: 0.0,
                texture_paths: Vec::new(),
                is_instance: false,
            })
            .collect();

        // Emit a portable `.phx` sidecar for the USD file.
        Self::write_phx_sidecar(path, &meshes);

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

    /// Write `<usd>.phx` next to the source USD file (best-effort, freshness-gated).
    pub fn write_phx_sidecar(usd_path: &Path, meshes: &[crate::phx::Mesh]) {
        use crate::phx::exporters::scene::{
            PhxMaterial, PhxMesh, PhxScene, export_phx_scene_to_bytes,
        };

        let phx_path = usd_path.with_extension("phx");
        let should_write =
            if let (Ok(p), Ok(b)) = (std::fs::metadata(&phx_path), std::fs::metadata(usd_path)) {
                p.modified().unwrap() < b.modified().unwrap()
            } else {
                true
            };

        if !should_write {
            println!("[USD] .phx sidecar up to date: {:?}", phx_path);
            return;
        }

        let mut phx = PhxScene::new(&usd_path.file_stem().unwrap_or_default().to_string_lossy());
        for m in meshes {
            if m.positions.is_empty() {
                continue;
            }
            let colors = m
                .colors
                .clone()
                .unwrap_or_else(|| vec![[0.8, 0.8, 0.8, 1.0]; m.positions.len()]);
            let transform_matrix = m
                .transform
                .unwrap_or(glam::Mat4::IDENTITY)
                .to_cols_array_2d();
            phx.add_mesh(PhxMesh {
                name: m.name.clone(),
                vertices: m.positions.clone(),
                normals: m.normals.clone().unwrap_or_default(),
                uvs: m.uvs.clone().unwrap_or_default(),
                colors,
                indices: m.indices.clone(),
                materials: vec![format!("{}_mat", m.name)],
                matrix_world: transform_matrix,
                visible: true,
            });
            phx.add_material(PhxMaterial {
                name: format!("{}_mat", m.name),
                base_color: [0.8, 0.8, 0.8, 1.0],
                metallic: 0.1,
                roughness: 0.8,
                emission: [0.0, 0.0, 0.0],
                emission_strength: 0.0,
                alpha: 1.0,
                use_nodes: true,
                base_color_texture: None,
                normal_texture: None,
                roughness_texture: None,
            });
        }

        match export_phx_scene_to_bytes(&phx) {
            Ok(bytes) => match std::fs::write(&phx_path, &bytes) {
                Ok(_) => println!(
                    "[USD] wrote .phx sidecar ({} meshes, {} bytes): {:?}",
                    phx.meshes.len(),
                    bytes.len(),
                    phx_path
                ),
                Err(e) => eprintln!("[USD] failed to write .phx: {}", e),
            },
            Err(e) => eprintln!("[USD] failed to serialize .phx: {:?}", e),
        }
    }

    /// Simple USDA mesh for testing.
    pub fn sample_usda() -> &'static str {
        r#"#usda 1.0
(
    defaultPrim = "Mesh"
)

def Mesh "Mesh"
{
    float3[] extent = [(-0.5, -0.5, -0.5), (0.5, 0.5, 0.5)]
    int[] faceVertexCounts = [3, 3, 3, 3]
    int[] faceVertexIndices = [0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 1]
    point3f[] points = [(0, 0, 0.5), (0.5, 0, -0.5), (0, 0.5, -0.5), (-0.5, 0, -0.5), (0, -0.5, -0.5)]
    color3f[] primvars:displayColor = [(1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 0), (1, 0, 1)]
    vector3f[] primvars:normals = [(0, 0, 1), (0, 0, 1), (0, 0, 1), (0, 0, 1), (0, 0, 1)]
    uniform token subdivisionScheme = "none"
}
"#
    }
}

#[cfg(test)]
mod tests;
