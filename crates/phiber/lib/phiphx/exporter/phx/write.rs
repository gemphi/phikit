//! PHX point cloud chunk and binary writer.

use crate::phx::exporters::blend::chunks::PhxChunkType;
use crate::phx::{PointCloud, Result};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

/// PHX point cloud chunk
#[derive(Debug, Clone)]
pub struct PhxPointCloudChunk {
    /// Cloud name
    pub name: String,
    /// Positions (flattened [x,y,z] per point)
    pub positions: Vec<f32>,
    /// Normals (optional)
    pub normals: Option<Vec<f32>>,
    /// Colors (optional, [r,g,b,a] per point)
    pub colors: Option<Vec<f32>>,
}

impl PhxPointCloudChunk {
    /// Create from PointCloud
    pub fn from_point_cloud(cloud: &PointCloud) -> Self {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut colors = Vec::new();

        let has_normals = cloud.vertices().iter().any(|v| v.normal.is_some());
        let has_colors = cloud.vertices().iter().any(|v| v.color.is_some());

        for vertex in cloud.vertices() {
            positions.push(vertex.position[0]);
            positions.push(vertex.position[1]);
            positions.push(vertex.position[2]);

            if has_normals {
                if let Some(n) = vertex.normal {
                    normals.push(n[0]);
                    normals.push(n[1]);
                    normals.push(n[2]);
                } else {
                    normals.push(0.0);
                    normals.push(0.0);
                    normals.push(0.0);
                }
            }

            if has_colors {
                if let Some(c) = vertex.color {
                    colors.push(c[0] as f32 / 255.0);
                    colors.push(c[1] as f32 / 255.0);
                    colors.push(c[2] as f32 / 255.0);
                    colors.push(c[3] as f32 / 255.0);
                } else {
                    colors.push(1.0);
                    colors.push(1.0);
                    colors.push(1.0);
                    colors.push(1.0);
                }
            }
        }

        Self {
            name: cloud.name.clone(),
            positions,
            normals: if has_normals { Some(normals) } else { None },
            colors: if has_colors { Some(colors) } else { None },
        }
    }

    /// Write point cloud chunk to bytes
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Chunk type
        writer.write_u32::<LittleEndian>(PhxChunkType::PointCloud as u32)?;

        // Name
        let name_bytes = self.name.as_bytes();
        let name_len = name_bytes.len().min(255);
        writer.write_u8(name_len as u8)?;
        writer.write_all(&name_bytes[..name_len])?;
        writer.write_u8(0)?;

        // Point count
        let point_count = self.positions.len() / 3;
        writer.write_u32::<LittleEndian>(point_count as u32)?;

        // Flags
        let mut flags: u32 = 0;
        if self.normals.is_some() {
            flags |= 0x01;
        }
        if self.colors.is_some() {
            flags |= 0x02;
        }
        writer.write_u32::<LittleEndian>(flags)?;

        // Positions
        for &val in &self.positions {
            writer.write_f32::<LittleEndian>(val)?;
        }

        // Normals
        if let Some(ref normals) = self.normals {
            for &val in normals {
                writer.write_f32::<LittleEndian>(val)?;
            }
        }

        // Colors
        if let Some(ref colors) = self.colors {
            for &val in colors {
                writer.write_f32::<LittleEndian>(val)?;
            }
        }

        Ok(())
    }
}
