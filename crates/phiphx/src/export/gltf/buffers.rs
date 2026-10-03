//! glTF buffer/accessor/bufferView assembly.
//!
//! Constants and the [`GltfBufferWriter`] helper that packs vertex/index data
//! into a single binary buffer alongside the matching JSON metadata.

use byteorder::{LittleEndian, WriteBytesExt};
use serde_json::{Value, json};

/// glTF buffer target constants
pub const ARRAY_BUFFER: u32 = 34962;
pub const ELEMENT_ARRAY_BUFFER: u32 = 34963;

/// glTF component types
pub const COMPONENT_TYPE_FLOAT: u32 = 5126;
pub const COMPONENT_TYPE_UNSIGNED_SHORT: u32 = 5123;

/// glTF primitive modes
pub const PRIMITIVE_MODE_POINTS: u32 = 0;
pub const PRIMITIVE_MODE_TRIANGLES: u32 = 4;

/// Helper to write binary glTF buffer data
#[derive(Debug, Default)]
pub struct GltfBufferWriter {
    pub(crate) buffers: Vec<Value>,
    pub(crate) buffer_views: Vec<Value>,
    pub(crate) accessors: Vec<Value>,
    pub(crate) current_buffer: Vec<u8>,
}

impl GltfBufferWriter {
    /// Create a new buffer writer
    pub fn new() -> Self {
        Self::default()
    }

    /// Pad buffer to alignment
    pub(crate) fn pad_to_align(&mut self, align: usize) {
        while self.current_buffer.len() % align != 0 {
            self.current_buffer.push(0);
        }
    }

    /// Write float32 array and return accessor index
    pub fn write_f32_array(&mut self, data: &[f32], target: u32) -> usize {
        self.pad_to_align(4);
        let offset = self.current_buffer.len();

        for &val in data {
            self.current_buffer.write_f32::<LittleEndian>(val).unwrap();
        }

        // Create buffer view
        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": data.len() * 4,
            "target": target
        }));

        // Determine component type and count
        let count = if data.len() % 3 == 0 {
            data.len() / 3
        } else if data.len() % 2 == 0 {
            data.len() / 2
        } else {
            data.len()
        };

        let type_str = if data.len() % 3 == 0 {
            "VEC3"
        } else if data.len() % 2 == 0 {
            "VEC2"
        } else {
            "SCALAR"
        };

        // Calculate min/max
        let (min_vals, max_vals) = if type_str == "VEC3" {
            let min_vals: Vec<f32> = (0..3)
                .map(|i| {
                    data.iter()
                        .skip(i)
                        .step_by(3)
                        .cloned()
                        .fold(f32::INFINITY, f32::min)
                })
                .collect();
            let max_vals: Vec<f32> = (0..3)
                .map(|i| {
                    data.iter()
                        .skip(i)
                        .step_by(3)
                        .cloned()
                        .fold(f32::NEG_INFINITY, f32::max)
                })
                .collect();
            (min_vals, max_vals)
        } else if type_str == "VEC2" {
            let min_vals: Vec<f32> = (0..2)
                .map(|i| {
                    data.iter()
                        .skip(i)
                        .step_by(2)
                        .cloned()
                        .fold(f32::INFINITY, f32::min)
                })
                .collect();
            let max_vals: Vec<f32> = (0..2)
                .map(|i| {
                    data.iter()
                        .skip(i)
                        .step_by(2)
                        .cloned()
                        .fold(f32::NEG_INFINITY, f32::max)
                })
                .collect();
            (min_vals, max_vals)
        } else {
            let min_val = data.iter().cloned().fold(f32::INFINITY, f32::min);
            let max_val = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            (vec![min_val], vec![max_val])
        };

        let acc_idx = self.accessors.len();
        self.accessors.push(json!({
            "bufferView": bv_idx,
            "componentType": COMPONENT_TYPE_FLOAT,
            "count": count,
            "type": type_str,
            "min": min_vals,
            "max": max_vals
        }));

        acc_idx
    }

    /// Write uint16 indices and return accessor index
    pub fn write_u16_array(&mut self, data: &[u16], target: u32) -> usize {
        self.pad_to_align(2);
        let offset = self.current_buffer.len();

        for &val in data {
            self.current_buffer.write_u16::<LittleEndian>(val).unwrap();
        }

        let bv_idx = self.buffer_views.len();
        self.buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": data.len() * 2,
            "target": target
        }));

        let acc_idx = self.accessors.len();
        self.accessors.push(json!({
            "bufferView": bv_idx,
            "componentType": COMPONENT_TYPE_UNSIGNED_SHORT,
            "count": data.len(),
            "type": "SCALAR"
        }));

        acc_idx
    }

    /// Finalize buffer and return binary data
    pub fn finalize(mut self) -> (Vec<u8>, Vec<Value>, Vec<Value>, Vec<Value>) {
        self.buffers
            .push(json!({"byteLength": self.current_buffer.len()}));
        (
            self.current_buffer,
            self.buffers,
            self.buffer_views,
            self.accessors,
        )
    }
}
