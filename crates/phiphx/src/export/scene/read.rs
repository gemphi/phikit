//! PHX scene binary deserialization.

use super::types::{PHX_SCENE_MAGIC, PhxScene};
use crate::{PointCloudError, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

impl PhxScene {
/// Read a PHX scene (magic, JSON length, JSON).
pub fn read<R: Read>(reader: &mut R) -> Result<PhxScene> {
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

/// Read a PHX scene from bytes.
pub fn from_bytes(data: &[u8]) -> Result<PhxScene> {
    let mut reader = data;
    Self::read(&mut reader)
}
}
