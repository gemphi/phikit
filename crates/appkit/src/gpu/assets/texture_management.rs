//! USD/USDA/USDZ and PHX asset loading with texture and color management.
//!
//! Handles complex mesh formats that involve vertex color processing,
//! tint baking, and point cloud color conversion.

use crate::gpu::{MeshBuffer, Vertex};
use std::path::Path;
use super::asset_loading::AssetLoader;

impl AssetLoader {
    /// Load a USD/USDA/USDZ mesh using native phx importer.
    ///
    /// Handles USDC binary meshes (common in Apple's BOTAnist .usdz files):
    /// - Calls `import_file_as_mesh` to extract Mesh structs from USD data
    /// - Converts Mesh positions/normals/colors to GPU Vertex arrays
    /// - The 0.01 scale factor for USDC (Apple centimeters → meters) is applied
    ///   by the importer's `import_usdc_as_mesh` internally
    /// - Generates face normals when the importer returns None for normals
    /// - Logs AABB warning when bounding box is empty after extraction
    /// - `tint` is baked into vertex colors so the scene base_color reaches the shader
    pub fn load_usda(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        path: &Path,
        entity_id: u64,
        tint: [f32; 4],
    ) -> Result<MeshBuffer, String> {
        println!("[AssetLoader] Loading USD: {:?}", path);

        let path_str = path.to_string_lossy().to_string();
        let meshes = phx::importers::usd::UsdImporter::import_file_as_mesh(&path_str)
            .map_err(|e| format!("USD import failed: {:?}", e))?;

        if meshes.is_empty() {
            return Err("USD file contains no meshes".to_string());
        }

        // Determine if this is a USDC-based file (USDZ containing USDC, or raw .usdc).
        // The importer already applies 0.01 scale for USDC binary data internally
        // (Apple Reality Composer uses centimeters). We detect the format here for logging.
        let is_usdc_based = {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if ext == "usdc" {
                true
            } else if ext == "usdz" {
                // USDZ files from Apple BOTAnist typically contain USDC payloads
                // The importer handles scale internally via import_usdc_as_mesh
                if let Ok(data) = std::fs::read(path) {
                    // Check if it's a zip containing USDC
                    data.len() >= 4 && data[0] == 0x50 && data[1] == 0x4B
                } else {
                    false
                }
            } else {
                false
            }
        };

        if is_usdc_based {
            println!(
                "[AssetLoader] USDC-based file detected — 0.01 scale factor applied by importer"
            );
        }

        // Combine all USD meshes into one MeshBuffer
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut needs_normals = false;

        // Use tint as vertex color multiplier; fall back to white if tint is near-zero
        let tint_color = if tint[0] + tint[1] + tint[2] < 0.01 {
            [1.0f32, 1.0, 1.0, 1.0]
        } else {
            tint
        };

        for mesh in &meshes {
            let vertex_offset = all_vertices.len() as u32;

            let normals = mesh.normals.as_ref();
            let colors = mesh.colors.as_ref();

            // Track whether this mesh has no normals (importer returned None)
            if normals.is_none() {
                needs_normals = true;
            }

            for (i, pos) in mesh.positions.iter().enumerate() {
                let normal = normals
                    .and_then(|n| n.get(i))
                    .copied()
                    .unwrap_or([0.0, 0.0, 0.0]); // Zero = will trigger flat normal computation

                let mesh_color = colors
                    .and_then(|c| c.get(i))
                    .copied()
                    .unwrap_or([1.0, 1.0, 1.0, 1.0]); // White = neutral

                // Bake tint into vertex color so base_color reaches the shader
                // even if the material uniform bind group is stale
                let color = [
                    mesh_color[0] * tint_color[0],
                    mesh_color[1] * tint_color[1],
                    mesh_color[2] * tint_color[2],
                    mesh_color[3] * tint_color[3],
                ];

                all_vertices.push(Vertex {
                    position: *pos,
                    normal,
                    color,
                });
            }

            for tri in &mesh.indices {
                all_indices.push(vertex_offset + tri[0]);
                all_indices.push(vertex_offset + tri[1]);
                all_indices.push(vertex_offset + tri[2]);
            }
        }

        println!(
            "[AssetLoader] USD loaded: {} vertices, {} indices from {} meshes",
            all_vertices.len(),
            all_indices.len(),
            meshes.len()
        );
        if !all_indices.is_empty() {
            println!(
                "[AssetLoader] First 3 indices: {}, {}, {}",
                all_indices[0], all_indices[1], all_indices[2]
            );
            if !all_vertices.is_empty() {
                println!("[AssetLoader] First Vertex: {:?}", all_vertices[0].position);
            }
        }

        // Generate face normals when the importer returns None for normals,
        // or when all extracted normals are zero (degenerate data).
        if needs_normals || all_vertices.iter().all(|v| v.normal == [0.0, 0.0, 0.0]) {
            println!("[AssetLoader] Generating face normals (importer returned None or all-zero normals)");
            Self::compute_flat_normals(&mut all_vertices, &all_indices);
        }

        // Compute AABB and log warning if empty (indicates vertex extraction failure)
        let (aabb_min, aabb_max) = Self::compute_aabb(&all_vertices);
        let aabb_size = [
            aabb_max[0] - aabb_min[0],
            aabb_max[1] - aabb_min[1],
            aabb_max[2] - aabb_min[2],
        ];
        if aabb_size[0].abs() < f32::EPSILON
            && aabb_size[1].abs() < f32::EPSILON
            && aabb_size[2].abs() < f32::EPSILON
        {
            println!("[AssetLoader] WARNING: Mesh AABB is empty! Vertex extraction might be failing. path={:?}", path);
        }

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
}
