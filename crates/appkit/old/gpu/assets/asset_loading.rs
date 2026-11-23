//! Core asset loading dispatch, OBJ format support, and geometry utilities.
//!
//! Contains the `AssetLoader` struct, the unified `load_mesh` dispatcher,
//! OBJ loading, and shared geometry helpers (AABB, flat normals).

use crate::gpu::{MeshBuffer, Vertex};
use std::path::Path;

/// Central asset loader — dispatches mesh loading by file extension.
pub struct AssetLoader;

impl AssetLoader {
    /// Unified mesh loader — dispatches by file extension.
    /// `tint` is baked into vertex colors so the base_color reaches the shader
    /// even if the material uniform bind group is stale.
    pub fn load_mesh(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        path: &Path,
        entity_id: u64,
        tint: [f32; 4],
    ) -> Result<MeshBuffer, String> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "obj" => Self::load_obj(device, layout, path, entity_id, tint),
            "usda" | "usd" | "usdz" | "usdc" => {
                Self::load_usda(device, layout, path, entity_id, tint)
            }
            "ply" => Self::load_phx_point_cloud(device, layout, path, entity_id),
            "phx" => Self::load_phx(device, layout, path, entity_id),
            "blend" => Self::load_blend(device, layout, path, entity_id),
            _ => {
                println!(
                    "[AssetLoader] Unsupported format '{}', trying OBJ fallback",
                    ext
                );
                Self::load_obj(device, layout, path, entity_id, tint)
            }
        }
    }

    /// Load an OBJ mesh from disk.
    /// `tint` is multiplied into vertex colors so the scene base_color is baked in.
    pub fn load_obj(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        path: &Path,
        entity_id: u64,
        tint: [f32; 4],
    ) -> Result<MeshBuffer, String> {
        println!("[AssetLoader] Loading OBJ: {:?}", path);

        let load_options = tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        };

        let (models, _materials) = tobj::load_obj(path, &load_options)
            .map_err(|e| format!("Failed to load OBJ: {}", e))?;

        if models.is_empty() {
            return Err("OBJ file contains no models".to_string());
        }

        // Use tint as vertex color; fall back to white if tint is near-zero
        let color = if tint[0] + tint[1] + tint[2] < 0.01 {
            [1.0, 1.0, 1.0, 1.0]
        } else {
            tint
        };

        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();

        for model in models {
            let mesh = &model.mesh;
            let vertex_offset = all_vertices.len() as u32;

            for i in 0..mesh.positions.len() / 3 {
                let pos = [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                ];

                let normal = if !mesh.normals.is_empty() {
                    [
                        mesh.normals[i * 3],
                        mesh.normals[i * 3 + 1],
                        mesh.normals[i * 3 + 2],
                    ]
                } else {
                    [0.0, 0.0, 0.0]
                };

                all_vertices.push(Vertex {
                    position: pos,
                    normal,
                    color,
                });
            }

            for idx in &mesh.indices {
                all_indices.push(vertex_offset + *idx);
            }
        }

        println!(
            "[AssetLoader] OBJ loaded: {} vertices, {} indices, tint=[{:.2},{:.2},{:.2},{:.2}]",
            all_vertices.len(),
            all_indices.len(),
            color[0],
            color[1],
            color[2],
            color[3]
        );

        Ok(MeshBuffer::new(
            device,
            layout,
            &all_vertices,
            Some(&all_indices),
            crate::gpu::mesh::MaterialUniform::default(),
            path.to_string_lossy().to_string(),
            entity_id,
        ))
    }

    /// Compute axis-aligned bounding box from vertices.
    /// Returns (min, max) corners. If vertices is empty, returns zeros.
    pub(crate) fn compute_aabb(vertices: &[Vertex]) -> ([f32; 3], [f32; 3]) {
        if vertices.is_empty() {
            return ([0.0; 3], [0.0; 3]);
        }
        let mut bmin = [f32::MAX; 3];
        let mut bmax = [f32::MIN; 3];
        for v in vertices {
            for i in 0..3 {
                bmin[i] = bmin[i].min(v.position[i]);
                bmax[i] = bmax[i].max(v.position[i]);
            }
        }
        (bmin, bmax)
    }

    /// Compute flat normals from triangle indices
    pub(crate) fn compute_flat_normals(vertices: &mut [Vertex], indices: &[u32]) {
        for v in vertices.iter_mut() {
            v.normal = [0.0, 0.0, 0.0];
        }
        for tri in indices.chunks(3) {
            if tri.len() < 3 {
                continue;
            }
            let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
            if i0 >= vertices.len() || i1 >= vertices.len() || i2 >= vertices.len() {
                continue;
            }
            let p0 = vertices[i0].position;
            let p1 = vertices[i1].position;
            let p2 = vertices[i2].position;
            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            for &idx in &[i0, i1, i2] {
                vertices[idx].normal[0] += n[0];
                vertices[idx].normal[1] += n[1];
                vertices[idx].normal[2] += n[2];
            }
        }
        for v in vertices.iter_mut() {
            let len =
                (v.normal[0] * v.normal[0] + v.normal[1] * v.normal[1] + v.normal[2] * v.normal[2])
                    .sqrt();
            if len > 0.0001 {
                v.normal[0] /= len;
                v.normal[1] /= len;
                v.normal[2] /= len;
            } else {
                v.normal = [0.0, 1.0, 0.0]; // Default up
            }
        }
    }
}
