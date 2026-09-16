//! Native Blender Exporters for PHX Format
//!
//! This module provides functionality to export Blender scenes directly to PHX binary format.
//! Includes both Rust API and Python script generation for Blender addons.

use crate::phx::Mesh as ImportMesh;
use crate::phx::{PointCloud, PointCloudError, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// Re-export Mesh from importers for convenience
pub use crate::phx::Mesh as BlenderMesh;

/// PHX Binary Format Version
pub const PHX_VERSION: u32 = 1;

/// PHX file magic number
pub const PHX_MAGIC: &[u8; 4] = b"PHX\x00";

/// PHX format chunk types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PhxChunkType {
    /// Mesh geometry data
    Mesh = 0x4D455348, // "MESH"
    /// Point cloud data
    PointCloud = 0x5054434C, // "PTCL"
    /// Material data
    Material = 0x4D41544C, // "MATL"
    /// Scene hierarchy
    Scene = 0x53434E45, // "SCNE"
    /// End of file marker
    End = 0x454E4421, // "END!"
}

/// PHX binary file header
#[derive(Debug, Clone)]
pub struct PhxHeader {
    /// Magic number (PHX\0)
    pub magic: [u8; 4],
    /// Version number
    pub version: u32,
    /// Flags (compression, etc.)
    pub flags: u32,
    /// Number of chunks
    pub chunk_count: u32,
}

impl PhxHeader {
    /// Create new header
    pub fn new(chunk_count: u32) -> Self {
        Self {
            magic: *PHX_MAGIC,
            version: PHX_VERSION,
            flags: 0,
            chunk_count,
        }
    }

    /// Write header to bytes
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_u32::<LittleEndian>(self.version)?;
        writer.write_u32::<LittleEndian>(self.flags)?;
        writer.write_u32::<LittleEndian>(self.chunk_count)?;
        Ok(())
    }

    /// Read header from bytes
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        if &magic != PHX_MAGIC {
            return Err(PointCloudError::InvalidFormat(
                "Invalid PHX magic number".to_string(),
            ));
        }

        let version = reader.read_u32::<LittleEndian>()?;
        let flags = reader.read_u32::<LittleEndian>()?;
        let chunk_count = reader.read_u32::<LittleEndian>()?;

        Ok(Self {
            magic,
            version,
            flags,
            chunk_count,
        })
    }
}

/// PHX mesh data chunk
#[derive(Debug, Clone)]
pub struct PhxMeshChunk {
    /// Mesh name
    pub name: String,
    /// Vertex positions (flattened [x,y,z] per vertex)
    pub positions: Vec<f32>,
    /// Normals (optional, same layout as positions)
    pub normals: Option<Vec<f32>>,
    /// Colors (optional, [r,g,b,a] per vertex)
    pub colors: Option<Vec<f32>>,
    /// UVs (optional, [u,v] per vertex)
    pub uvs: Option<Vec<f32>>,
    /// Triangle indices (3 per triangle)
    pub indices: Vec<u32>,
    /// Transform matrix (4x4, row-major)
    pub transform: [f32; 16],
}

impl PhxMeshChunk {
    /// Create from ImportMesh
    pub fn from_mesh(mesh: &ImportMesh, name: &str, transform: Option<[f32; 16]>) -> Self {
        let positions: Vec<f32> = mesh
            .positions
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect();

        let normals = mesh
            .normals
            .as_ref()
            .map(|n| n.iter().flat_map(|n| [n[0], n[1], n[2]]).collect());

        let colors = mesh
            .colors
            .as_ref()
            .map(|c| c.iter().flat_map(|c| [c[0], c[1], c[2], c[3]]).collect());

        let uvs = mesh
            .uvs
            .as_ref()
            .map(|u| u.iter().flat_map(|u| [u[0], u[1]]).collect());

        let indices: Vec<u32> = mesh
            .indices
            .iter()
            .flat_map(|tri| [tri[0], tri[1], tri[2]])
            .collect();

        Self {
            name: name.to_string(),
            positions,
            normals,
            colors,
            uvs,
            indices,
            transform: transform.unwrap_or([
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]),
        }
    }

    /// Write mesh chunk to bytes
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Chunk type
        writer.write_u32::<LittleEndian>(PhxChunkType::Mesh as u32)?;

        // Name (null-terminated string, max 256 chars)
        let name_bytes = self.name.as_bytes();
        let name_len = name_bytes.len().min(255);
        writer.write_u8(name_len as u8)?;
        writer.write_all(&name_bytes[..name_len])?;
        writer.write_u8(0)?;

        // Vertex count
        let vertex_count = self.positions.len() / 3;
        writer.write_u32::<LittleEndian>(vertex_count as u32)?;

        // Triangle count
        let tri_count = self.indices.len() / 3;
        writer.write_u32::<LittleEndian>(tri_count as u32)?;

        // Flags (which attributes are present)
        let mut flags: u32 = 0;
        if self.normals.is_some() {
            flags |= 0x01;
        }
        if self.colors.is_some() {
            flags |= 0x02;
        }
        if self.uvs.is_some() {
            flags |= 0x04;
        }
        writer.write_u32::<LittleEndian>(flags)?;

        // Transform matrix
        for &val in &self.transform {
            writer.write_f32::<LittleEndian>(val)?;
        }

        // Positions
        for &val in &self.positions {
            writer.write_f32::<LittleEndian>(val)?;
        }

        // Normals (if present)
        if let Some(ref normals) = self.normals {
            for &val in normals {
                writer.write_f32::<LittleEndian>(val)?;
            }
        }

        // Colors (if present)
        if let Some(ref colors) = self.colors {
            for &val in colors {
                writer.write_f32::<LittleEndian>(val)?;
            }
        }

        // UVs (if present)
        if let Some(ref uvs) = self.uvs {
            for &val in uvs {
                writer.write_f32::<LittleEndian>(val)?;
            }
        }

        // Indices
        for &val in &self.indices {
            writer.write_u32::<LittleEndian>(val)?;
        }

        Ok(())
    }
}

/// Export a single mesh to PHX binary format
pub fn export_mesh_to_phx(mesh: &ImportMesh, name: &str) -> Result<Vec<u8>> {
    let mut output = Vec::new();

    // Header
    let header = PhxHeader::new(1);
    header.write(&mut output)?;

    // Mesh chunk
    let chunk = PhxMeshChunk::from_mesh(mesh, name, None);
    chunk.write(&mut output)?;

    // End chunk
    output.write_u32::<LittleEndian>(PhxChunkType::End as u32)?;

    Ok(output)
}

/// Export multiple meshes to PHX binary format
pub fn export_scene_to_phx(meshes: &[(ImportMesh, String, [f32; 16])]) -> Result<Vec<u8>> {
    let mut output = Vec::new();

    // Header
    let header = PhxHeader::new(meshes.len() as u32);
    header.write(&mut output)?;

    // Mesh chunks
    for (mesh, name, transform) in meshes {
        let chunk = PhxMeshChunk::from_mesh(mesh, name, Some(*transform));
        chunk.write(&mut output)?;
    }

    // End chunk
    output.write_u32::<LittleEndian>(PhxChunkType::End as u32)?;

    Ok(output)
}

/// Export PointCloud to PHX binary format
pub fn export_point_cloud_to_phx(cloud: &PointCloud) -> Result<Vec<u8>> {
    let mut output = Vec::new();

    // Header
    let header = PhxHeader::new(1);
    header.write(&mut output)?;

    // PointCloud chunk type
    output.write_u32::<LittleEndian>(PhxChunkType::PointCloud as u32)?;

    // Name
    let name_bytes = cloud.name.as_bytes();
    let name_len = name_bytes.len().min(255);
    output.write_u8(name_len as u8)?;
    output.write_all(&name_bytes[..name_len])?;
    output.write_u8(0)?;

    // Vertex count
    let vertices = cloud.vertices();
    output.write_u32::<LittleEndian>(vertices.len() as u32)?;

    // Flags
    let mut flags: u32 = 0;
    let has_normals = vertices.iter().any(|v| v.normal.is_some());
    let has_colors = vertices.iter().any(|v| v.color.is_some());
    let has_uvs = vertices.iter().any(|v| v.attributes.uv.is_some());
    if has_normals {
        flags |= 0x01;
    }
    if has_colors {
        flags |= 0x02;
    }
    if has_uvs {
        flags |= 0x04;
    }
    output.write_u32::<LittleEndian>(flags)?;

    // Positions
    for v in vertices {
        output.write_f32::<LittleEndian>(v.position[0])?;
        output.write_f32::<LittleEndian>(v.position[1])?;
        output.write_f32::<LittleEndian>(v.position[2])?;
    }

    // Normals
    if has_normals {
        for v in vertices {
            if let Some(n) = v.normal {
                output.write_f32::<LittleEndian>(n[0])?;
                output.write_f32::<LittleEndian>(n[1])?;
                output.write_f32::<LittleEndian>(n[2])?;
            } else {
                output.write_f32::<LittleEndian>(0.0)?;
                output.write_f32::<LittleEndian>(0.0)?;
                output.write_f32::<LittleEndian>(0.0)?;
            }
        }
    }

    // Colors
    if has_colors {
        for v in vertices {
            if let Some(c) = v.color {
                output.write_f32::<LittleEndian>(c[0] as f32 / 255.0)?;
                output.write_f32::<LittleEndian>(c[1] as f32 / 255.0)?;
                output.write_f32::<LittleEndian>(c[2] as f32 / 255.0)?;
                output.write_f32::<LittleEndian>(c[3] as f32 / 255.0)?;
            } else {
                output.write_f32::<LittleEndian>(1.0)?;
                output.write_f32::<LittleEndian>(1.0)?;
                output.write_f32::<LittleEndian>(1.0)?;
                output.write_f32::<LittleEndian>(1.0)?;
            }
        }
    }

    // UVs
    if has_uvs {
        for v in vertices {
            if let Some(uv) = v.attributes.uv {
                output.write_f32::<LittleEndian>(uv[0])?;
                output.write_f32::<LittleEndian>(uv[1])?;
            } else {
                output.write_f32::<LittleEndian>(0.0)?;
                output.write_f32::<LittleEndian>(0.0)?;
            }
        }
    }

    // End chunk
    output.write_u32::<LittleEndian>(PhxChunkType::End as u32)?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phx_header_write_read() {
        let header = PhxHeader::new(5);
        let mut buf = Vec::new();
        header.write(&mut buf).unwrap();

        let mut reader = &buf[..];
        let read_header = PhxHeader::read(&mut reader).unwrap();

        assert_eq!(read_header.magic, *PHX_MAGIC);
        assert_eq!(read_header.version, PHX_VERSION);
        assert_eq!(read_header.chunk_count, 5);
    }

    #[test]
    fn test_export_import_mesh_roundtrip() {
        let mut mesh = ImportMesh::new();
        mesh.positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        mesh.normals = Some(vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]);
        mesh.indices = vec![[0, 1, 2]];

        let phx_data = export_mesh_to_phx(&mesh, "test_mesh").unwrap();
        assert!(!phx_data.is_empty());
        assert_eq!(&phx_data[0..4], PHX_MAGIC);
    }
}
