//! `GltfImporter` and the `Importer` trait implementation.

use crate::phx::importer::Importer;
use crate::phx::{Mesh, MeshToCloudConfig};
use crate::phx::{PointCloud, PointCloudError, Result};

/// glTF importer
pub struct GltfImporter;

impl GltfImporter {
    /// Check if data is a valid glTF file
    pub fn can_import(data: &[u8]) -> bool {
        // Check for glTF JSON magic
        if data.len() >= 4 {
            // GLB magic: "glTF" = 0x46546C67
            if data[0] == 0x67 && data[1] == 0x6C && data[2] == 0x54 && data[3] == 0x46 {
                return true;
            }
        }

        // Check for JSON glTF (starts with '{' and contains "asset")
        if data.starts_with(b"{") {
            // Quick check for asset field
            if let Ok(text) = std::str::from_utf8(&data[..data.len().min(1000)]) {
                return text.contains("\"asset\"") && text.contains("\"assetVersion\"")
                    || text.contains("\"asset\"") && text.contains("\"version\"");
            }
        }

        // Check for GLB binary chunk
        if data.len() >= 12 {
            // GLB header: magic (4) + version (4) + length (4)
            let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            if magic == 0x46546C67 {
                // "glTF"
                return true;
            }
        }

        false
    }

    /// Import glTF from bytes using the gltf crate
    pub fn import_bytes(data: &[u8]) -> Result<PointCloud> {
        let config = MeshToCloudConfig::default();
        Self::import_bytes_with_config(data, &config)
    }

    /// Import with custom configuration
    pub fn import_bytes_with_config(data: &[u8], config: &MeshToCloudConfig) -> Result<PointCloud> {
        // Use the gltf crate to parse
        let gltf = gltf::Gltf::from_slice(data)
            .map_err(|e| PointCloudError::ImportError(format!("glTF parse error: {}", e)))?;

        // Collect all meshes from all scenes
        let mut combined_mesh = Mesh::new();

        for scene in gltf.scenes() {
            for node in scene.nodes() {
                let mesh_ref: Option<gltf::Mesh<'_>> = node.mesh();
                if let Some(mesh) = mesh_ref {
                    let node_transform = Self::extract_transform(&node);
                    let mesh_data = Self::extract_mesh(&mesh, &node_transform)?;

                    // Merge into combined mesh
                    let vertex_offset = combined_mesh.vertex_count() as u32;
                    combined_mesh
                        .positions
                        .extend_from_slice(&mesh_data.positions);

                    if let Some(ref normals) = mesh_data.normals {
                        let combined = combined_mesh.normals.get_or_insert_with(Vec::new);
                        combined.extend_from_slice(normals);
                    }

                    if let Some(ref colors) = mesh_data.colors {
                        let combined = combined_mesh.colors.get_or_insert_with(Vec::new);
                        combined.extend_from_slice(colors);
                    }

                    for tri in &mesh_data.indices {
                        combined_mesh.indices.push([
                            tri[0] + vertex_offset,
                            tri[1] + vertex_offset,
                            tri[2] + vertex_offset,
                        ]);
                    }
                }
            }
        }

        if combined_mesh.triangle_count() == 0 {
            // Try to extract point clouds from glTF directly
            return Self::extract_point_cloud(&gltf, config);
        }

        Ok(combined_mesh.to_point_cloud(config))
    }

    /// Import from file path
    pub fn import_file(path: &str) -> Result<PointCloud> {
        let config = MeshToCloudConfig::default();
        Self::import_file_with_config(path, &config)
    }

    /// Import from file with custom configuration
    pub fn import_file_with_config(path: &str, config: &MeshToCloudConfig) -> Result<PointCloud> {
        let mut cloud = Self::import_full(path, config)?;
        cloud.metadata.source_path = Some(path.to_string());
        cloud.metadata.source_format = Some("glTF".to_string());

        Ok(cloud)
    }

    /// Import a glTF/GLB file into a single combined triangle [`Mesh`].
    pub fn import_combined_mesh(path: &str) -> Result<Mesh> {
        super::full::import_gltf_combined_mesh(path)
    }

    /// Import a glTF/GLB file and extract its per-node meshes (unmerged).
    pub fn import_meshes(path: &str) -> Result<Vec<Mesh>> {
        super::full::import_gltf_meshes(path)
    }

    /// Full glTF import that loads buffers and converts to a point cloud.
    pub fn import_full(path: &str, config: &MeshToCloudConfig) -> Result<PointCloud> {
        super::full::import_gltf_full(path, config)
    }
}

impl Importer for GltfImporter {
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
