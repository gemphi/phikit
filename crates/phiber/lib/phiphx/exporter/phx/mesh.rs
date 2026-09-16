//! PHX mesh chunk reader.

use crate::phx::Result;
use crate::phx::exporters::blend::chunks::PhxMeshChunk;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

pub(super) fn read_mesh_chunk<R: Read>(reader: &mut R) -> Result<PhxMeshChunk> {
    // Name
    let name_len = reader.read_u8()? as usize;
    let mut name_bytes = vec![0u8; name_len];
    reader.read_exact(&mut name_bytes)?;
    let name = String::from_utf8_lossy(&name_bytes).to_string();
    let _null = reader.read_u8()?;

    // Vertex count
    let vertex_count = reader.read_u32::<LittleEndian>()? as usize;

    // Triangle count
    let tri_count = reader.read_u32::<LittleEndian>()? as usize;

    // Flags
    let flags = reader.read_u32::<LittleEndian>()?;
    let has_normals = flags & 0x01 != 0;
    let has_colors = flags & 0x02 != 0;
    let has_uvs = flags & 0x04 != 0;

    // Transform
    let mut transform = [0f32; 16];
    for i in 0..16 {
        transform[i] = reader.read_f32::<LittleEndian>()?;
    }

    // Positions
    let mut positions = Vec::with_capacity(vertex_count * 3);
    for _ in 0..vertex_count * 3 {
        positions.push(reader.read_f32::<LittleEndian>()?);
    }

    // Normals
    let normals = if has_normals {
        let mut n = Vec::with_capacity(vertex_count * 3);
        for _ in 0..vertex_count * 3 {
            n.push(reader.read_f32::<LittleEndian>()?);
        }
        Some(n)
    } else {
        None
    };

    // Colors
    let colors = if has_colors {
        let mut c = Vec::with_capacity(vertex_count * 4);
        for _ in 0..vertex_count * 4 {
            c.push(reader.read_f32::<LittleEndian>()?);
        }
        Some(c)
    } else {
        None
    };

    // UVs
    let uvs = if has_uvs {
        let mut u = Vec::with_capacity(vertex_count * 2);
        for _ in 0..vertex_count * 2 {
            u.push(reader.read_f32::<LittleEndian>()?);
        }
        Some(u)
    } else {
        None
    };

    // Indices
    let mut indices = Vec::with_capacity(tri_count * 3);
    for _ in 0..tri_count * 3 {
        indices.push(reader.read_u32::<LittleEndian>()?);
    }

    Ok(PhxMeshChunk {
        name,
        positions,
        normals,
        colors,
        uvs,
        indices,
        transform,
    })
}
