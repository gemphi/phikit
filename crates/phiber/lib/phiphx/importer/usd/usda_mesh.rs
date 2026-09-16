//! USDA (ASCII) → full mesh parser (preserves triangles and transforms).

use super::UsdImporter;
use crate::phx::Mesh;
use crate::phx::{PointCloudError, Result};

impl UsdImporter {
    /// Import USDA as full Meshes (preserves triangles)
    pub(crate) fn import_usda_as_mesh(text: &str) -> Result<Vec<Mesh>> {
        let mut meshes = Vec::new();
        let mut current_name = "UnknownMesh".to_string();
        let mut current_positions: Vec<[f32; 3]> = Vec::new();
        let mut current_normals: Vec<[f32; 3]> = Vec::new();
        let mut current_colors: Vec<[f32; 4]> = Vec::new();
        let mut current_counts: Vec<u32> = Vec::new();
        let mut current_indices: Vec<u32> = Vec::new();

        // Track transformation hierarchy: (depth, matrix)
        let mut transform_stack: Vec<(i32, glam::Mat4)> = Vec::new();
        let mut in_transform_attr = false;
        let mut current_transform_floats: Vec<f32> = Vec::new();

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

            // Count braces and pop transforms
            for c in trimmed.chars() {
                match c {
                    '{' => brace_depth += 1,
                    '}' => {
                        // Pop transforms if we are exiting their scope
                        while let Some(&(depth, _)) = transform_stack.last() {
                            if depth >= brace_depth {
                                transform_stack.pop();
                            } else {
                                break;
                            }
                        }
                        brace_depth -= 1;
                    }
                    _ => {}
                }
            }

            // Detect transform definition
            if trimmed.starts_with("matrix4d xformOp:transform") {
                in_transform_attr = true;
                current_transform_floats.clear();
            }

            if in_transform_attr {
                Self::accumulate_transform_floats(trimmed, &mut current_transform_floats);

                if current_transform_floats.len() >= 16 {
                    in_transform_attr = false;

                    // Construct glam::Mat4 (USD uses row-major, glam uses column-major)
                    let m = &current_transform_floats;
                    let local_transform = glam::Mat4::from_cols_array(&[
                        m[0], m[4], m[8], m[12], m[1], m[5], m[9], m[13], m[2], m[6], m[10], m[14],
                        m[3], m[7], m[11], m[15],
                    ]);

                    // Multiply with parent if exists
                    let parent_transform = transform_stack
                        .last()
                        .map(|(_, t)| *t)
                        .unwrap_or(glam::Mat4::IDENTITY);
                    let global_transform = parent_transform * local_transform;

                    transform_stack.push((brace_depth, global_transform));
                }
            }

            // Detect mesh definition
            if trimmed.starts_with("def Mesh") || trimmed.starts_with("def Point") {
                in_mesh = true;
                mesh_open_depth = None;

                // Extract name
                if let Some(start) = trimmed.find('"') {
                    if let Some(end) = trimmed[start + 1..].find('"') {
                        current_name = trimmed[start + 1..start + 1 + end].to_string();
                    }
                }

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
                Self::accumulate_int(&mut in_counts_attr, &mut current_counts, trimmed);
                Self::accumulate_int(&mut in_indices_attr, &mut current_indices, trimmed);

                // End of mesh
                if let Some(open_depth) = mesh_open_depth {
                    if trimmed.contains('}') && brace_depth < open_depth {
                        in_mesh = false;
                        mesh_open_depth = None;

                        Self::push_usda_mesh(
                            &mut meshes,
                            &current_name,
                            &current_positions,
                            &current_normals,
                            &current_counts,
                            &current_indices,
                            transform_stack.last().map(|&(_, t)| t),
                        );
                    }
                }
            }
        }

        if meshes.is_empty() {
            return Err(PointCloudError::ImportError(
                "No meshes found in USDA".to_string(),
            ));
        }

        Ok(meshes)
    }

    /// Parse the floats out of a `matrix4d xformOp:transform` line fragment.
    ///
    /// Mirrors the original inline scanner: characters that make up the
    /// attribute keyword/punctuation act as separators, and runs of digits
    /// (with `.`, `-`, `e`/`E`) are parsed as `f32` values.
    fn accumulate_transform_floats(trimmed: &str, out: &mut Vec<f32>) {
        let mut current_num = String::new();
        for c in trimmed.chars() {
            match c {
                '(' | ')' | ',' | ' ' | '\t' | '\r' | '\n' | '[' | ']' | '=' | 'm' | 'a' | 't'
                | 'r' | 'i' | 'x' | '4' | 'd' | 'f' | 'o' | 'O' | 'p' | ':' | 's' => {
                    if !current_num.is_empty() {
                        if let Ok(n) = current_num.parse::<f32>() {
                            out.push(n);
                        }
                        current_num.clear();
                    }
                }
                c if c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E' => {
                    current_num.push(c);
                }
                _ => {}
            }
        }
        if !current_num.is_empty() {
            if let Ok(n) = current_num.parse::<f32>() {
                out.push(n);
            }
        }
    }
}
