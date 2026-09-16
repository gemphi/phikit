//! PHX scene binary deserialization.

use super::types::{PHX_SCENE_MAGIC, PhxScene};
use crate::phx::{PointCloudError, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

/// Read PHX scene from reader
pub fn read_scene<R: Read>(reader: &mut R) -> Result<PhxScene> {
    // Read and verify magic
    let mut magic = [0u8; 10];
    reader.read_exact(&mut magic)?;
    if &magic != PHX_SCENE_MAGIC {
        return Err(PointCloudError::InvalidFormat(
            "Invalid PHX scene magic number".to_string(),
        ));
    }

    // Read JSON length
    let json_len = reader.read_u32::<LittleEndian>()? as usize;

    // Read JSON data
    let mut json_bytes = vec![0u8; json_len];
    reader.read_exact(&mut json_bytes)?;

    // Deserialize
    let scene: PhxScene = serde_json::from_slice(&json_bytes)
        .map_err(|e| PointCloudError::Serialization(e.to_string()))?;

    Ok(scene)
}

/// Import PHX scene from bytes
pub fn import_scene_from_bytes(data: &[u8]) -> Result<PhxScene> {
    let mut reader = data;
    read_scene(&mut reader)
}
