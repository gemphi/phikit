//! `.usd` / `.usda` / `.usdc` / `.usdz` format importer
//!
//! Converts Universal Scene Description files to point clouds and meshes.
//! Handles USDA (ASCII), USDC (binary crate), and USDZ (zip container).

use std::path::Path;

pub mod exporter;
pub mod importer;
mod parse;
mod scene;
mod usda;
mod usda_build;
mod usda_mesh;
mod usda_parse;
mod usdc_fallback;

pub use exporter::UsdExporter;
pub use importer::UsdImporter;
pub use scene::UsdScene;

impl UsdImporter {
    /// Full USD import using external library when available.
    #[cfg(feature = "usdc-parser")]
    pub fn import_with_openusd(
        path: &str,
        config: &crate::MeshToCloudConfig,
    ) -> crate::Result<crate::PointCloud> {
        Self::import_file_with_config(path, config)
    }

    /// Unified import entry point — called by [`AssetImporter::import`].
    pub fn import(
        path: &Path,
        progress: &dyn Fn(f32, &str),
    ) -> Result<crate::AssetImportResult, String> {
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

        let objects: Vec<crate::ImportedObject> = meshes
            .iter()
            .map(|m| crate::ImportedObject {
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

        Ok(crate::AssetImportResult {
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
    pub fn write_phx_sidecar(usd_path: &Path, meshes: &[crate::Mesh]) {
        crate::device::phx::sidecar::write_for_usd(usd_path, meshes);
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

