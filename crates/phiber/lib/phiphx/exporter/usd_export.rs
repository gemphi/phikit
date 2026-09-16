//! USDA/USDC format exporter for point clouds.

use crate::phx::{PointCloud, PointCloudError, Result};

/// Implements USD point cloud export operations.
pub struct UsdExporter;

impl UsdExporter {
    /// Export PointCloud to USDA format (human-readable USD)
    pub fn export_usda(cloud: &PointCloud) -> Result<String> {
        let mut output = String::new();

        output.push_str("#usda 1.0\n");
        output.push_str("(\n");
        output.push_str("    doc = \"PHX Point Cloud Export\"\n");
        output.push_str(")\n");
        output.push('\n');

        output.push_str("def PointInstancer \"pointCloud\"\n");
        output.push_str("{\n");

        output.push_str("    point3f[] positions = [\n");
        let vertices: Vec<_> = cloud.vertices().iter().collect();
        for (i, vertex) in vertices.iter().enumerate() {
            let comma = if i < vertices.len() - 1 { "," } else { "" };
            output.push_str(&format!(
                "        ({:.6}, {:.6}, {:.6}){}\n",
                vertex.position[0], vertex.position[1], vertex.position[2], comma
            ));
        }
        output.push_str("    ]\n");

        let has_normals = vertices.iter().any(|v| v.normal.is_some());
        if has_normals {
            output.push_str("    normal3f[] normals = [\n");
            for (i, vertex) in vertices.iter().enumerate() {
                let n = vertex.normal.unwrap_or([0.0, 0.0, 1.0]);
                let comma = if i < vertices.len() - 1 { "," } else { "" };
                output.push_str(&format!(
                    "        ({:.6}, {:.6}, {:.6}){}\n",
                    n[0], n[1], n[2], comma
                ));
            }
            output.push_str("    ]\n");
        }

        let has_colors = vertices.iter().any(|v| v.color.is_some());
        if has_colors {
            output.push_str("    color3f[] primvars:displayColor = [\n");
            for (i, vertex) in vertices.iter().enumerate() {
                let c = vertex.color.unwrap_or([255, 255, 255, 255]);
                let comma = if i < vertices.len() - 1 { "," } else { "" };
                output.push_str(&format!(
                    "        ({:.3}, {:.3}, {:.3}){}\n",
                    c[0] as f32 / 255.0,
                    c[1] as f32 / 255.0,
                    c[2] as f32 / 255.0,
                    comma
                ));
            }
            output.push_str("    ]\n");
            output.push_str("    int[] primvars:displayColor:interpolation = [\"vertex\"]\n");
        }

        output.push('}');
        output.push('\n');

        Ok(output)
    }

    /// Export PointCloud to USDC format (binary USD)
    pub fn export_usdc(_cloud: &PointCloud) -> Result<Vec<u8>> {
        Err(PointCloudError::ExportError(
            "Binary USD (USDC) export not yet implemented. Use export_usda() for ASCII format."
                .to_string(),
        ))
    }
}
