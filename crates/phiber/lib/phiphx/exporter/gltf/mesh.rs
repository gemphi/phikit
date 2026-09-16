//! Triangle mesh glTF export.

use super::base64::base64_encode;
use super::buffers::{
    ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, GltfBufferWriter, PRIMITIVE_MODE_TRIANGLES,
};
use crate::phx::{PointCloudError, Result};
use byteorder::{LittleEndian, WriteBytesExt};
use serde_json::json;

/// Export mesh to glTF 2.0 format
pub fn export_mesh_gltf(
    positions: &[[f32; 3]],
    normals: Option<&[[f32; 3]]>,
    uvs: Option<&[[f32; 2]]>,
    indices: &[[u32; 3]],
) -> Result<String> {
    let mut writer = GltfBufferWriter::new();

    // Flatten positions
    let pos_flat: Vec<f32> = positions.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();
    let pos_acc = writer.write_f32_array(&pos_flat, ARRAY_BUFFER);

    // Build attributes
    let mut attributes = json!({"POSITION": pos_acc});

    // Add normals if present
    if let Some(norms) = normals {
        let norm_flat: Vec<f32> = norms.iter().flat_map(|n| [n[0], n[1], n[2]]).collect();
        let norm_acc = writer.write_f32_array(&norm_flat, ARRAY_BUFFER);
        attributes["NORMAL"] = json!(norm_acc);
    }

    // Add UVs if present
    if let Some(uv_data) = uvs {
        let uv_flat: Vec<f32> = uv_data.iter().flat_map(|u| [u[0], u[1]]).collect();
        let uv_acc = writer.write_f32_array(&uv_flat, ARRAY_BUFFER);
        attributes["TEXCOORD_0"] = json!(uv_acc);
    }

    // Flatten indices (convert u32 to u16 if possible, otherwise use u32)
    let idx_flat: Vec<u32> = indices.iter().flat_map(|i| [i[0], i[1], i[2]]).collect();
    let idx_acc = if idx_flat.iter().all(|&i| i <= u16::MAX as u32) {
        let idx_u16: Vec<u16> = idx_flat.iter().map(|&i| i as u16).collect();
        writer.write_u16_array(&idx_u16, ELEMENT_ARRAY_BUFFER)
    } else {
        // Use u32 - need to add custom accessor
        writer.pad_to_align(4);
        let offset = writer.current_buffer.len();
        for &val in &idx_flat {
            writer
                .current_buffer
                .write_u32::<LittleEndian>(val)
                .unwrap();
        }
        let bv_idx = writer.buffer_views.len();
        writer.buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": idx_flat.len() * 4,
            "target": ELEMENT_ARRAY_BUFFER
        }));
        let acc_idx = writer.accessors.len();
        writer.accessors.push(json!({
            "bufferView": bv_idx,
            "componentType": 5125u32, // UNSIGNED_INT
            "count": idx_flat.len(),
            "type": "SCALAR"
        }));
        acc_idx
    };

    // Build primitive
    let primitive = json!({
        "attributes": attributes,
        "indices": idx_acc,
        "mode": PRIMITIVE_MODE_TRIANGLES
    });

    // Finalize
    let (buffer_data, _buffers, buffer_views, accessors) = writer.finalize();

    let gltf = json!({
        "asset": {
            "version": "2.0",
            "generator": "PHX Native glTF Exporter"
        },
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "name": "Mesh"}],
        "meshes": [{
            "name": "Mesh",
            "primitives": [primitive]
        }],
        "buffers": [{
            "uri": format!("data:application/octet-stream;base64,{}" , base64_encode(&buffer_data)),
            "byteLength": buffer_data.len()
        }],
        "bufferViews": buffer_views,
        "accessors": accessors
    });

    serde_json::to_string_pretty(&gltf).map_err(|e| PointCloudError::Serialization(e.to_string()))
}
