//! Point cloud glTF/GLB export.

use super::base64::{base64_decode, base64_encode};
use super::buffers::{ARRAY_BUFFER, GltfBufferWriter, PRIMITIVE_MODE_POINTS};
use crate::phx::{PointCloud, PointCloudError, Result};
use byteorder::{LittleEndian, WriteBytesExt};
use serde_json::json;
use std::io::Write;

/// Export point cloud to glTF 2.0 format
pub fn export_point_cloud_gltf(cloud: &PointCloud, binary: bool) -> Result<String> {
    let mut writer = GltfBufferWriter::new();

    // Build position array
    let mut positions: Vec<f32> = Vec::new();
    let mut normals: Vec<f32> = Vec::new();
    let mut colors: Vec<f32> = Vec::new();

    let mut max_pos = [f32::NEG_INFINITY; 3];
    let mut min_pos = [f32::INFINITY; 3];

    for vertex in cloud.vertices() {
        let pos = vertex.position;
        positions.push(pos[0]);
        positions.push(pos[1]);
        positions.push(pos[2]);

        for i in 0..3 {
            max_pos[i] = max_pos[i].max(pos[i]);
            min_pos[i] = min_pos[i].min(pos[i]);
        }

        if let Some(n) = vertex.normal {
            normals.push(n[0]);
            normals.push(n[1]);
            normals.push(n[2]);
        }

        if let Some(c) = vertex.color {
            colors.push(c[0] as f32 / 255.0);
            colors.push(c[1] as f32 / 255.0);
            colors.push(c[2] as f32 / 255.0);
            colors.push(c[3] as f32 / 255.0);
        }
    }

    let _vertex_count = cloud.vertex_count();
    let has_normals = !normals.is_empty();
    let has_colors = !colors.is_empty();

    // Write to buffer
    let pos_acc = writer.write_f32_array(&positions, ARRAY_BUFFER);
    let mut attributes = json!({
        "POSITION": pos_acc
    });

    if has_normals {
        let norm_acc = writer.write_f32_array(&normals, ARRAY_BUFFER);
        attributes["NORMAL"] = json!(norm_acc);
    }

    if has_colors {
        let color_acc = writer.write_f32_array(&colors, ARRAY_BUFFER);
        attributes["COLOR_0"] = json!(color_acc);
    }

    // Build glTF structure
    let (buffer_data, buffers, buffer_views, accessors) = writer.finalize();

    let gltf = json!({
        "asset": {
            "version": "2.0",
            "generator": "PHX Native glTF Exporter"
        },
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "name": "PHXPointCloud"}],
        "meshes": [{
            "name": "PointCloud",
            "primitives": [{
                "attributes": attributes,
                "mode": PRIMITIVE_MODE_POINTS
            }]
        }],
        "buffers": buffers,
        "bufferViews": buffer_views,
        "accessors": accessors
    });

    if binary {
        // Binary glTF (GLB)
        let json_str = serde_json::to_string(&gltf)
            .map_err(|e| PointCloudError::Serialization(e.to_string()))?;
        let mut json_bytes = json_str.into_bytes();

        // JSON chunk padding
        let json_padding = (4 - (json_bytes.len() % 4)) % 4;
        json_bytes.extend(std::iter::repeat(b' ').take(json_padding));

        // BIN chunk padding
        let bin_padding = (4 - (buffer_data.len() % 4)) % 4;
        let bin_len = buffer_data.len() + bin_padding;

        // Header: 12 bytes
        let total_len = 12 + 8 + json_bytes.len() + 8 + bin_len;
        let mut glb = Vec::with_capacity(total_len);

        // Magic + version + length
        glb.write_u32::<LittleEndian>(total_len as u32).unwrap();
        glb.write_all(b"glTF").unwrap();
        glb.write_u32::<LittleEndian>(2).unwrap();

        // JSON chunk
        glb.write_u32::<LittleEndian>(json_bytes.len() as u32)
            .unwrap();
        glb.write_all(b"JSON").unwrap();
        glb.extend_from_slice(&json_bytes);

        // BIN chunk
        glb.write_u32::<LittleEndian>(bin_len as u32).unwrap();
        glb.write_all(b"BIN\0").unwrap();
        glb.extend_from_slice(&buffer_data);
        glb.extend(std::iter::repeat(0).take(bin_padding));

        // Return base64 encoded for string compatibility
        Ok(base64_encode(&glb))
    } else {
        // Separate files - return JSON with external buffer reference
        let mut gltf_with_uri = gltf;
        gltf_with_uri["buffers"][0]["uri"] = json!(format!(
            "data:application/octet-stream;base64,{}",
            base64_encode(&buffer_data)
        ));

        serde_json::to_string_pretty(&gltf_with_uri)
            .map_err(|e| PointCloudError::Serialization(e.to_string()))
    }
}

/// Export point cloud to binary glTF (GLB) bytes
pub fn export_point_cloud_glb(cloud: &PointCloud) -> Result<Vec<u8>> {
    let json_str = export_point_cloud_gltf(cloud, true)?;
    // The true flag returns base64, so decode it
    base64_decode(&json_str).map_err(|e| PointCloudError::Serialization(e))
}
