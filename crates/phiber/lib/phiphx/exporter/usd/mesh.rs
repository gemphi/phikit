//! Mesh/scene → USDA exporter

use super::UsdaWriter;
use crate::phx::Result;
use std::fmt::Write as FmtWrite;

/// Export mesh to USDA format
pub fn export_mesh_usda(
    name: &str,
    positions: &[[f32; 3]],
    normals: Option<&[[f32; 3]]>,
    uvs: Option<&[[f32; 2]]>,
    colors: Option<&[[f32; 4]]>,
    face_vertex_counts: &[i32],
    face_vertex_indices: &[i32],
    transform: Option<[[f32; 4]; 4]>,
) -> Result<String> {
    let mut writer = UsdaWriter::new();

    // Root Xform
    writer.push_scope("Root", "Xform");

    // Object Xform
    writer.push_scope(name, "Xform");

    if let Some(mat) = transform {
        writer.matrix_attr("xformOp:transform", &mat);
        // xformOpOrder is handled specially
        writer.line("uniform token[] xformOpOrder = [\"xformOp:transform\"]");
    }

    // Mesh
    writer.push_scope("mesh", "Mesh");

    // Points (positions)
    let mut points_str = String::new();
    for (_i, pos) in positions.iter().enumerate() {
        let comma = if _i < positions.len() - 1 { "," } else { "" };
        write!(
            points_str,
            "({:.6}, {:.6}, {:.6}){}, ",
            pos[0], pos[1], pos[2], comma
        )
        .unwrap();
    }
    // Remove trailing comma and space
    let points_str = points_str.trim_end_matches(", ");
    writer.line(&format!("point3f[] points = [{}]", points_str));

    // Face counts and indices
    let fvc_str: Vec<String> = face_vertex_counts.iter().map(|v| v.to_string()).collect();
    writer.line(&format!(
        "int[] faceVertexCounts = [{}]",
        fvc_str.join(", ")
    ));

    let fvi_str: Vec<String> = face_vertex_indices.iter().map(|v| v.to_string()).collect();
    writer.line(&format!(
        "int[] faceVertexIndices = [{}]",
        fvi_str.join(", ")
    ));

    // Normals
    if let Some(norms) = normals {
        let mut norms_str = String::new();
        for (i, n) in norms.iter().enumerate() {
            let comma = if i < norms.len() - 1 { "," } else { "" };
            write!(
                norms_str,
                "({:.6}, {:.6}, {:.6}){}, ",
                n[0], n[1], n[2], comma
            )
            .unwrap();
        }
        let norms_str = norms_str.trim_end_matches(", ");
        writer.line(&format!("vector3f[] primvars:normals = [{}]", norms_str));
        writer.line("uniform token[] primvars:normals:interpolation = [\"vertex\"]");
    }

    // UVs
    if let Some(uv_data) = uvs {
        let mut uv_str = String::new();
        for (i, uv) in uv_data.iter().enumerate() {
            let comma = if i < uv_data.len() - 1 { "," } else { "" };
            write!(uv_str, "({:.6}, {:.6}){}, ", uv[0], uv[1], comma).unwrap();
        }
        let uv_str = uv_str.trim_end_matches(", ");
        writer.line(&format!("texCoord2f[] primvars:st = [{}]", uv_str));
        writer.line("uniform token[] primvars:st:interpolation = [\"vertex\"]");
    }

    // Display colors
    if let Some(cols) = colors {
        let mut col_str = String::new();
        for (i, c) in cols.iter().enumerate() {
            let comma = if i < cols.len() - 1 { "," } else { "" };
            write!(
                col_str,
                "({:.6}, {:.6}, {:.6}){}, ",
                c[0], c[1], c[2], comma
            )
            .unwrap();
        }
        let col_str = col_str.trim_end_matches(", ");
        writer.line(&format!("color3f[] primvars:displayColor = [{}]", col_str));
        writer.line("uniform token[] primvars:displayColor:interpolation = [\"vertex\"]");
    }

    // Disable subdivision
    writer.line("uniform token subdivisionScheme = \"none\"");

    writer.pop_scope(); // mesh
    writer.pop_scope(); // object
    writer.pop_scope(); // Root

    Ok(writer.get_text())
}
