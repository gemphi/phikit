//! Mesh/cloud finalisation shared by the USDA parsers.
//!
//! Turns accumulated `positions`/`normals`/`colors` plus
//! `faceVertexCounts`/`faceVertexIndices` into either point-cloud vertices
//! or full `Mesh` records, preserving the original behaviour exactly.

use super::UsdImporter;
use crate::phx::{Mesh, MeshToCloudConfig};
use crate::phx::{PointCloud, Vertex};

impl UsdImporter {
    /// Build triangle indices from USD `faceVertexCounts`/`faceVertexIndices`.
    ///
    /// When counts are present each face is fan-triangulated; otherwise the
    /// indices are taken as a flat triangle list.
    pub(crate) fn build_face_indices(mesh: &mut Mesh, counts: &[u32], indices: &[u32]) {
        if !counts.is_empty() {
            let mut idx = 0usize;
            for &count in counts {
                let n = count as usize;
                if n < 3 || idx + n > indices.len() {
                    break;
                }
                let base = indices[idx];
                for k in 1..(n - 1) {
                    let i1 = indices[idx + k];
                    let i2 = indices[idx + k + 1];
                    mesh.indices.push([base, i1, i2]);
                }
                idx += n;
            }
        } else {
            for tri in indices.chunks(3) {
                if tri.len() == 3 {
                    mesh.indices.push([tri[0], tri[1], tri[2]]);
                }
            }
        }
    }

    /// Finalise an accumulated mesh into the point cloud (USDA → cloud path).
    pub(crate) fn finalize_usda_cloud(
        cloud: &mut PointCloud,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        colors: &[[f32; 4]],
        counts: &[u32],
        indices: &[u32],
        config: &MeshToCloudConfig,
    ) {
        // Convert to vertices
        if positions.is_empty() {
            return;
        }
        // If we have indices, build triangles
        if indices.len() >= 3 {
            // Build mesh and sample
            let mut mesh = Mesh::new();
            mesh.positions = positions.to_vec();
            if !normals.is_empty() {
                mesh.normals = Some(normals.to_vec());
            }
            if !colors.is_empty() {
                mesh.colors = Some(colors.to_vec());
            }

            // Build triangles from faceVertexIndices + faceVertexCounts when available.
            Self::build_face_indices(&mut mesh, counts, indices);

            let sampled = mesh.to_point_cloud(config);
            cloud.add_vertices(sampled.vertices().iter().copied());
        } else {
            // No indices - use positions directly as points
            for (i, pos) in positions.iter().enumerate() {
                let mut vertex = Vertex::new(pos[0], pos[1], pos[2]);

                if let Some(n) = normals.get(i) {
                    vertex = vertex.with_normal(n[0], n[1], n[2]);
                }

                if let Some(c) = colors.get(i) {
                    vertex = vertex.with_color(
                        (c[0] * 255.0) as u8,
                        (c[1] * 255.0) as u8,
                        (c[2] * 255.0) as u8,
                        ((c.get(3).copied().unwrap_or(1.0)) * 255.0) as u8,
                    );
                }

                cloud.add_vertex(vertex);
            }
        }
    }

    /// Finalise an accumulated mesh and push it onto `meshes` (USDA → mesh path).
    pub(crate) fn push_usda_mesh(
        meshes: &mut Vec<Mesh>,
        name: &str,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        counts: &[u32],
        indices: &[u32],
        transform: Option<glam::Mat4>,
    ) {
        // Build mesh with triangles
        if positions.is_empty() || indices.len() < 3 {
            return;
        }
        let mut mesh = Mesh::new();
        mesh.name = name.to_string();
        mesh.positions = positions.to_vec();

        // Assign transform if we have one in scope
        if let Some(transform) = transform {
            mesh.transform = Some(transform);
        }

        if !normals.is_empty() {
            mesh.normals = Some(normals.to_vec());
        }

        // Build triangles from faceVertexIndices + faceVertexCounts
        Self::build_face_indices(&mut mesh, counts, indices);

        meshes.push(mesh);
    }
}
