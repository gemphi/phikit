//! Full glTF import that loads buffers and converts to a point cloud.

use crate::phx::exporter::binary::BinaryExporter;
use crate::phx::{Mesh, MeshToCloudConfig};
use crate::phx::{PointCloud, PointCloudError, Result};

use super::mesh::GltfMeshExtractor;

/// Import a glTF/GLB file into a single combined triangle [`Mesh`], loading
/// every buffer (embedded base64, external `.bin`, or GLB blob) and merging
/// all scene meshes with their node transforms baked into positions.
///
/// This is the mesh-level entry point shared by [`import_gltf_full`] (which
/// samples the result into a point cloud) and by GPU consumers such as the
/// `apple` crate's `AssetLoader::load_gltf`, which converts the returned mesh
/// straight into a renderable vertex/index buffer — so glTF loading lives in
/// exactly one place rather than being re-implemented per consumer.
/// Load a glTF/GLB file and extract its per-node meshes (unmerged), loading
/// every buffer (embedded base64, external `.bin`, or GLB blob).
///
/// This is the per-object entry point used by [`import_gltf_combined_mesh`]
/// (which merges the result) and by the glTF scene wrapper (which resolves a
/// primary/active subject over the individual meshes).
pub(crate) fn import_gltf_meshes(path: &str) -> Result<Vec<Mesh>> {
    let data = std::fs::read(path)
        .map_err(|e| PointCloudError::ImportError(format!("Failed to read file: {}", e)))?;

    // Parse glTF
    let gltf = gltf::Gltf::from_slice(&data)
        .map_err(|e| PointCloudError::ImportError(format!("glTF parse error: {}", e)))?;

    // Load buffers
    let mut buffers: Vec<Vec<u8>> = Vec::new();

    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Uri(uri) => {
                if uri.starts_with("data:application/octet-stream;base64,") {
                    // Embedded base64 data URI
                    let base64_data = &uri["data:application/octet-stream;base64,".len()..];
                    let expected_len = buffer.length();
                    let decoded = BinaryExporter::base64_decode(base64_data, expected_len)
                        .map_err(|e| {
                            PointCloudError::ImportError(format!(
                                "Failed to decode base64 buffer: {}",
                                e
                            ))
                        })?;
                    buffers.push(decoded);
                } else {
                    // External buffer file
                    let base_path = std::path::Path::new(path)
                        .parent()
                        .map(|p| p.to_str().unwrap_or("."))
                        .unwrap_or(".");
                    let buffer_path = format!("{}/{}", base_path, uri);

                    let buffer_data = std::fs::read(&buffer_path).map_err(|e| {
                        PointCloudError::ImportError(format!(
                            "Failed to read buffer {}: {}",
                            uri, e
                        ))
                    })?;
                    buffers.push(buffer_data);
                }
            }
            gltf::buffer::Source::Bin => {
                // Embedded in GLB
                if let Some(blob) = gltf.blob.as_ref() {
                    buffers.push(blob.clone());
                } else {
                    buffers.push(Vec::new());
                }
            }
        }
    }

    GltfMeshExtractor::extract(&gltf, &buffers)
}

pub(crate) fn import_gltf_combined_mesh(path: &str) -> Result<Mesh> {
    // Extract the per-node meshes, then merge into one combined mesh.
    let meshes = import_gltf_meshes(path)?;
    let mut combined = Mesh::new();
    for mesh in &meshes {
        let offset = combined.vertex_count() as u32;
        combined.positions.extend_from_slice(&mesh.positions);

        if let Some(ref n) = mesh.normals {
            combined
                .normals
                .get_or_insert_with(Vec::new)
                .extend_from_slice(n);
        }
        if let Some(ref c) = mesh.colors {
            combined
                .colors
                .get_or_insert_with(Vec::new)
                .extend_from_slice(c);
        }

        for tri in &mesh.indices {
            combined
                .indices
                .push([tri[0] + offset, tri[1] + offset, tri[2] + offset]);
        }
    }
    combined.name = path.to_string();
    Ok(combined)
}

/// Full glTF import function that handles buffers and converts to a point cloud.
pub(crate) fn import_gltf_full(path: &str, config: &MeshToCloudConfig) -> Result<PointCloud> {
    let combined = import_gltf_combined_mesh(path)?;
    let mut cloud = combined.to_point_cloud(config);
    cloud.metadata.source_path = Some(path.to_string());
    cloud.metadata.source_format = Some("glTF".to_string());
    Ok(cloud)
}
