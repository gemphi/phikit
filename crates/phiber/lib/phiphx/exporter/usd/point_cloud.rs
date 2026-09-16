//! Point cloud → USDA exporters

use super::UsdaWriter;
use crate::phx::{PointCloud, PointCloudError, Result};
use std::fmt::Write as FmtWrite;

/// Export point cloud to USDA format
pub fn export_point_cloud_usda(cloud: &PointCloud) -> Result<String> {
    let mut writer = UsdaWriter::new();

    // Define point cloud prim
    writer.push_scope("PHXPointCloud", "PointInstancer");

    // Positions
    let vertices: Vec<_> = cloud.vertices().iter().collect();
    let mut positions_str = String::new();
    for (i, vertex) in vertices.iter().enumerate() {
        let comma = if i < vertices.len() - 1 { "," } else { "" };
        writeln!(
            positions_str,
            "        ({:.6}, {:.6}, {:.6}){}",
            vertex.position[0], vertex.position[1], vertex.position[2], comma
        )
        .unwrap();
    }
    writer.line("point3f[] positions = [");
    writer.lines.push(positions_str);
    writer.line("    ]");

    // Normals (if present)
    let has_normals = vertices.iter().any(|v| v.normal.is_some());
    if has_normals {
        let mut normals_str = String::new();
        for (i, vertex) in vertices.iter().enumerate() {
            let n = vertex.normal.unwrap_or([0.0, 0.0, 1.0]);
            let comma = if i < vertices.len() - 1 { "," } else { "" };
            writeln!(
                normals_str,
                "        ({:.6}, {:.6}, {:.6}){}",
                n[0], n[1], n[2], comma
            )
            .unwrap();
        }
        writer.line("normal3f[] normals = [");
        writer.lines.push(normals_str);
        writer.line("    ]");
    }

    // Colors (if present)
    let has_colors = vertices.iter().any(|v| v.color.is_some());
    if has_colors {
        let mut colors_str = String::new();
        for (i, vertex) in vertices.iter().enumerate() {
            let c = vertex.color.unwrap_or([255, 255, 255, 255]);
            let comma = if i < vertices.len() - 1 { "," } else { "" };
            writeln!(
                colors_str,
                "        ({:.3}, {:.3}, {:.3}){}",
                c[0] as f32 / 255.0,
                c[1] as f32 / 255.0,
                c[2] as f32 / 255.0,
                comma
            )
            .unwrap();
        }
        writer.line("color3f[] primvars:displayColor = [");
        writer.lines.push(colors_str);
        writer.line("    ]");
        writer.line("uniform token[] primvars:displayColor:interpolation = [\"vertex\"]");
    }

    // Proto indices (one per point)
    let proto_indices: Vec<String> = (0..vertices.len()).map(|i| i.to_string()).collect();
    writer.line(&format!(
        "int[] protoIndices = [{}]",
        proto_indices.join(", ")
    ));

    writer.pop_scope();

    Ok(writer.get_text())
}

/// Export point cloud as point instancer with sphere prototype
pub fn export_point_cloud_usda_with_spheres(
    cloud: &PointCloud,
    sphere_radius: f64,
) -> Result<String> {
    let mut writer = UsdaWriter::new();
    writer.push_scope("Root", "Xform");

    // PointInstancer
    writer.push_scope("PHXPointCloud", "PointInstancer");

    let vertices: Vec<_> = cloud.vertices().iter().collect();

    // Positions
    let mut pts_str = String::new();
    for (i, v) in vertices.iter().enumerate() {
        let comma = if i < vertices.len() - 1 { "," } else { "" };
        write!(
            pts_str,
            "({:.6}, {:.6}, {:.6}){}",
            v.position[0], v.position[1], v.position[2], comma
        )
        .unwrap();
    }
    writer.line(&format!("point3f[] positions = [{}]", pts_str));

    // Scales (all 1.0)
    let scales: Vec<String> = (0..vertices.len())
        .map(|_| "(1.0, 1.0, 1.0)".to_string())
        .collect();
    writer.line(&format!("vector3f[] scales = [{}]", scales.join(", ")));

    // Proto indices
    let ids: Vec<String> = (0..vertices.len()).map(|_i| "0".to_string()).collect();
    writer.line(&format!("int64[] protoIndices = [{}]", ids.join(", ")));

    // Prototypes reference
    writer.line("prepend rel prototypes = [</Sphere>]");

    writer.pop_scope();

    // Sphere prototype
    writer.push_scope("Sphere", "Sphere");
    writer.attr("double radius", &sphere_radius.to_string(), None);
    writer.pop_scope();

    writer.pop_scope(); // Root

    Ok(writer.get_text())
}

/// Export to USDC format (binary USD)
pub fn export_usdc(_cloud: &PointCloud) -> Result<Vec<u8>> {
    // Binary USD format is complex (Crate format)
    // For now, return an error - use USDA for compatibility
    Err(PointCloudError::ExportError(
        "Binary USD (USDC) export not yet implemented. Use export_point_cloud_usda() for ASCII format.".to_string()
    ))
}
