//! Mesh and point-cloud extraction from glTF documents.

use crate::phx::{Mesh, MeshToCloudConfig};
use crate::phx::{PointCloud, PointCloudError, Result};

use super::importer::GltfImporter;
use super::transform::transform_point;

impl GltfImporter {
    /// Extract mesh data from glTF primitive
    pub(crate) fn extract_mesh(gltf_mesh: &gltf::Mesh, _transform: &[f32; 16]) -> Result<Mesh> {
        let _mesh = Mesh::new();

        for primitive in gltf_mesh.primitives() {
            // Get positions
            let _reader = primitive.reader(|_buffer| {
                // This would normally read from the buffer cache
                // For now, return None since we need full buffer data
                None::<&[u8]>
            });

            // Since we can't easily get the buffer data without full file reading,
            // we'll need to re-parse with buffer data
            // This is a simplified space

            // In a full implementation, we would:
            // 1. Read positions from POSITION accessor
            // 2. Read normals from NORMAL accessor (optional)
            // 3. Read colors from COLOR_0 accessor (optional)
            // 4. Read indices from indices accessor
            // 5. Transform all positions by the node transform

            // For now, return an empty mesh
            // The actual implementation would use the gltf crate's full buffer reading
        }

        // Since we can't easily extract without full buffer access in this simplified version,
        // return an error indicating we need the buffer data
        Err(PointCloudError::ImportError(
            "Full glTF import requires buffer data access. Use import_file() instead.".to_string(),
        ))
    }

    /// Extract point cloud from glTF (for point-based primitives)
    pub(crate) fn extract_point_cloud(
        _gltf: &gltf::Gltf,
        _config: &MeshToCloudConfig,
    ) -> Result<PointCloud> {
        // glTF can have POINTS primitives
        // This would extract them directly

        let cloud = PointCloud::new("gltf_points");

        // In a full implementation:
        // 1. Look for primitives with mode = POINTS
        // 2. Extract POSITION accessor data
        // 3. Extract COLOR_0 if available
        // 4. Apply node transforms

        // Since we need buffer data access, return empty or error
        if cloud.is_empty() {
            return Err(PointCloudError::ImportError(
                "No point data found in glTF file (requires full buffer access)".to_string(),
            ));
        }

        Ok(cloud)
    }
}

/// Simple glTF mesh extractor that works with full file data
pub struct GltfMeshExtractor;

impl GltfMeshExtractor {
    /// Extract meshes from glTF document with full buffer data
    pub fn extract(gltf: &gltf::Gltf, buffers: &[Vec<u8>]) -> Result<Vec<Mesh>> {
        let mut meshes = Vec::new();

        for scene in gltf.scenes() {
            for node in scene.nodes() {
                if let Some(mesh) = node.mesh() {
                    let mesh_data = Self::extract_single_mesh(&mesh, buffers, &node)?;
                    meshes.push(mesh_data);
                }
            }
        }

        Ok(meshes)
    }

    /// Extract a single mesh
    fn extract_single_mesh(
        gltf_mesh: &gltf::Mesh,
        buffers: &[Vec<u8>],
        node: &gltf::Node,
    ) -> Result<Mesh> {
        let mut mesh = Mesh::new();
        // Derive a stable, deterministic name from the node/mesh indices so the
        // scene wrapper's name-based tie-break is meaningful. (The gltf crate's
        // string names require the `names` feature, which is not enabled here.)
        mesh.name = format!("node{}_mesh{}", node.index(), gltf_mesh.index());
        let transform = GltfImporter::extract_transform(node);

        let mut vertex_offset = 0u32;

        for primitive in gltf_mesh.primitives() {
            // Get reader with buffer data
            let reader =
                primitive.reader(|buffer| buffers.get(buffer.index()).map(|v| v.as_slice()));

            // Read positions
            if let Some(positions) = reader.read_positions() {
                // Transform and add positions
                for pos in positions {
                    let transformed = transform_point(&pos, &transform);
                    mesh.positions.push(transformed);
                }
            }

            // Read normals
            if let Some(normals) = reader.read_normals() {
                let normal_vec: Vec<[f32; 3]> = normals.collect();
                mesh.normals.get_or_insert_with(Vec::new).extend(normal_vec);
            }

            // Read colors
            if let Some(colors) = reader.read_colors(0) {
                let color_vec: Vec<[f32; 4]> = colors.into_rgba_f32().collect();
                mesh.colors.get_or_insert_with(Vec::new).extend(color_vec);
            }

            // Read indices
            if let Some(indices) = reader.read_indices() {
                let indices: Vec<u32> = indices.into_u32().collect();

                // Group into triangles
                for tri in indices.chunks(3) {
                    if tri.len() == 3 {
                        mesh.indices.push([
                            tri[0] + vertex_offset,
                            tri[1] + vertex_offset,
                            tri[2] + vertex_offset,
                        ]);
                    }
                }
            } else {
                // Non-indexed primitive - generate indices
                let pos_count = mesh.positions.len() - vertex_offset as usize;
                for i in (0..pos_count).step_by(3) {
                    mesh.indices.push([
                        vertex_offset + i as u32,
                        vertex_offset + (i + 1) as u32,
                        vertex_offset + (i + 2) as u32,
                    ]);
                }
            }

            vertex_offset = mesh.positions.len() as u32;
        }

        Ok(mesh)
    }
}
