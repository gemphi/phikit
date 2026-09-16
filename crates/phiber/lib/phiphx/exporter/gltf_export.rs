//! glTF 2.0 format exporter for point clouds.

use crate::phx::{PointCloud, PointCloudError, Result};
use serde_json::json;

/// Implements glTF point cloud export operations.
pub struct GltfExporter;

impl GltfExporter {
    /// Export PointCloud to glTF format
    pub fn export_gltf(cloud: &PointCloud, binary: bool) -> Result<String> {
        let mut positions: Vec<f32> = Vec::new();
        let mut normals: Vec<f32> = Vec::new();
        let mut colors: Vec<f32> = Vec::new();

        for vertex in cloud.vertices() {
            positions.push(vertex.position[0]);
            positions.push(vertex.position[1]);
            positions.push(vertex.position[2]);

            if let Some(n) = vertex.normal {
                normals.push(n[0]);
                normals.push(n[1]);
                normals.push(n[2]);
            }

            if let Some(c) = vertex.color {
                colors.push(c[0] as f32 / 255.0);
                colors.push(c[1] as f32 / 255.0);
                colors.push(c[2] as f32 / 255.0);
            }
        }

        let vertex_count = cloud.vertex_count();

        let gltf = json!({
            "asset": {
                "version": "2.0",
                "generator": "PHX Point Cloud Exporter"
            },
            "scene": 0,
            "scenes": [{"nodes": [0]}],
            "nodes": [{"mesh": 0}],
            "meshes": [{
                "primitives": [{
                    "mode": 0,  // POINTS
                    "attributes": {
                        "POSITION": 0
                    }
                }]
            }],
            "buffers": [{"byteLength": positions.len() * 4}],
            "bufferViews": [{
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": positions.len() * 4,
                "target": 34962  // ARRAY_BUFFER
            }],
            "accessors": [{
                "bufferView": 0,
                "byteOffset": 0,
                "componentType": 5126,  // FLOAT
                "count": vertex_count,
                "type": "VEC3",
                "max": [
                    positions.iter().step_by(3).cloned().fold(f32::NEG_INFINITY, f32::max),
                    positions.iter().skip(1).step_by(3).cloned().fold(f32::NEG_INFINITY, f32::max),
                    positions.iter().skip(2).step_by(3).cloned().fold(f32::NEG_INFINITY, f32::max)
                ],
                "min": [
                    positions.iter().step_by(3).cloned().fold(f32::INFINITY, f32::min),
                    positions.iter().skip(1).step_by(3).cloned().fold(f32::INFINITY, f32::min),
                    positions.iter().skip(2).step_by(3).cloned().fold(f32::INFINITY, f32::min)
                ]
            }]
        });

        let json_str = serde_json::to_string_pretty(&gltf)
            .map_err(|e| PointCloudError::Serialization(e.to_string()))?;

        if binary {
            // Pack binary buffer placeholder
            Ok(json_str)
        } else {
            Ok(json_str)
        }
    }
}
