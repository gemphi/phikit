//! USDA (ASCII) → point cloud parser.

use super::UsdImporter;
use crate::phx::MeshToCloudConfig;
use crate::phx::{PointCloud, Result};

impl UsdImporter {
    /// Import USDA (ASCII) format
    pub(crate) fn import_usda(text: &str, config: &MeshToCloudConfig) -> Result<PointCloud> {
        // Parse USDA format
        // This is a simplified parser for basic mesh data

        let mut cloud = PointCloud::new("usd_import");
        let mut current_positions: Vec<[f32; 3]> = Vec::new();
        let mut current_normals: Vec<[f32; 3]> = Vec::new();
        let mut current_colors: Vec<[f32; 4]> = Vec::new();
        let mut current_counts: Vec<u32> = Vec::new();
        let mut current_indices: Vec<u32> = Vec::new();

        let mut in_mesh = false;
        let mut in_points_attr = false;
        let mut in_normals_attr = false;
        let mut in_colors_attr = false;
        let mut in_counts_attr = false;
        let mut in_indices_attr = false;

        let mut brace_depth: i32 = 0;
        let mut mesh_open_depth: Option<i32> = None;

        for line in text.lines() {
            let trimmed = line.trim();

            // Count braces
            for c in trimmed.chars() {
                match c {
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    _ => {}
                }
            }

            // Detect mesh definition
            if trimmed.starts_with("def Mesh") || trimmed.starts_with("def Point") {
                in_mesh = true;
                mesh_open_depth = None;
                current_positions.clear();
                current_normals.clear();
                current_colors.clear();
                current_counts.clear();
                current_indices.clear();
            }

            // Record the depth where the mesh block actually opens.
            if in_mesh && mesh_open_depth.is_none() && trimmed.contains('{') {
                mesh_open_depth = Some(brace_depth);
            }

            // Detect attributes
            if in_mesh {
                Self::set_attr_flags(
                    trimmed,
                    &mut in_points_attr,
                    &mut in_normals_attr,
                    &mut in_colors_attr,
                    &mut in_counts_attr,
                    &mut in_indices_attr,
                );

                // Parse array data
                Self::accumulate_vec3(&mut in_points_attr, &mut current_positions, trimmed);
                Self::accumulate_vec3(&mut in_normals_attr, &mut current_normals, trimmed);
                Self::accumulate_color(&mut in_colors_attr, &mut current_colors, trimmed);
                Self::accumulate_int(&mut in_indices_attr, &mut current_indices, trimmed);
                Self::accumulate_int(&mut in_counts_attr, &mut current_counts, trimmed);

                // End of mesh: the closing brace takes us above the mesh's opening depth.
                if let Some(open_depth) = mesh_open_depth {
                    if trimmed.contains('}') && brace_depth < open_depth {
                        in_mesh = false;
                        mesh_open_depth = None;

                        Self::finalize_usda_cloud(
                            &mut cloud,
                            &current_positions,
                            &current_normals,
                            &current_colors,
                            &current_counts,
                            &current_indices,
                            config,
                        );
                    }
                }
            }
        }

        Ok(cloud)
    }
}
