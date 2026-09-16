//! PHX binary reader: import a point cloud from PHX data.

use super::mesh::read_mesh_chunk;
use crate::phx::exporters::blend::chunks::{PHX_VERSION, PhxChunkType, PhxHeader};
use crate::phx::{PointCloud, PointCloudError, Result, Vertex};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

/// Read PHX file and return point cloud
pub fn import_point_cloud_from_phx(data: &[u8]) -> Result<PointCloud> {
    let mut reader = data;

    // Read header
    let header = PhxHeader::read(&mut reader)?;
    if header.version != PHX_VERSION {
        return Err(PointCloudError::InvalidFormat(format!(
            "Unsupported PHX version: {}",
            header.version
        )));
    }

    let mut cloud = PointCloud::new("imported");

    // Read chunks
    for _ in 0..header.chunk_count {
        let chunk_type = reader.read_u32::<LittleEndian>()?;

        if chunk_type == PhxChunkType::PointCloud as u32 {
            read_point_cloud_chunk(&mut reader, &mut cloud)?;
        } else if chunk_type == PhxChunkType::Mesh as u32 {
            // Convert mesh to point cloud (sample from surface)
            let mesh = read_mesh_chunk(&mut reader)?;
            // Add mesh vertices as points
            for (i, pos) in mesh.positions.chunks_exact(3).enumerate() {
                let mut vertex = Vertex::new(pos[0], pos[1], pos[2]);

                if let Some(ref normals) = mesh.normals {
                    let n = &normals[i * 3..i * 3 + 3];
                    vertex = vertex.with_normal(n[0], n[1], n[2]);
                }

                if let Some(ref colors) = mesh.colors {
                    let c = &colors[i * 4..i * 4 + 4];
                    vertex = vertex.with_color(
                        (c[0] * 255.0) as u8,
                        (c[1] * 255.0) as u8,
                        (c[2] * 255.0) as u8,
                        (c[3] * 255.0) as u8,
                    );
                }

                cloud.add_vertex(vertex);
            }
        } else if chunk_type == PhxChunkType::End as u32 {
            break;
        }
    }

    Ok(cloud)
}

fn read_point_cloud_chunk<R: Read>(reader: &mut R, cloud: &mut PointCloud) -> Result<()> {
    // Name
    let name_len = reader.read_u8()? as usize;
    let mut name_bytes = vec![0u8; name_len];
    reader.read_exact(&mut name_bytes)?;
    let _name = String::from_utf8_lossy(&name_bytes);
    let _null = reader.read_u8()?; // null terminator

    // Point count
    let point_count = reader.read_u32::<LittleEndian>()? as usize;

    // Flags
    let flags = reader.read_u32::<LittleEndian>()?;
    let has_normals = flags & 0x01 != 0;
    let has_colors = flags & 0x02 != 0;

    // Read positions
    let mut positions = Vec::with_capacity(point_count * 3);
    for _ in 0..point_count * 3 {
        positions.push(reader.read_f32::<LittleEndian>()?);
    }

    // Read normals
    let mut normals = Vec::with_capacity(point_count * 3);
    if has_normals {
        for _ in 0..point_count * 3 {
            normals.push(reader.read_f32::<LittleEndian>()?);
        }
    }

    // Read colors
    let mut colors = Vec::with_capacity(point_count * 4);
    if has_colors {
        for _ in 0..point_count * 4 {
            colors.push(reader.read_f32::<LittleEndian>()?);
        }
    }

    // Build vertices
    for i in 0..point_count {
        let mut vertex = Vertex::new(positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]);

        if has_normals {
            vertex = vertex.with_normal(normals[i * 3], normals[i * 3 + 1], normals[i * 3 + 2]);
        }

        if has_colors {
            vertex = vertex.with_color(
                (colors[i * 4] * 255.0) as u8,
                (colors[i * 4 + 1] * 255.0) as u8,
                (colors[i * 4 + 2] * 255.0) as u8,
                (colors[i * 4 + 3] * 255.0) as u8,
            );
        }

        cloud.add_vertex(vertex);
    }

    Ok(())
}
