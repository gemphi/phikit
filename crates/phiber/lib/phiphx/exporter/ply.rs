//! PLY (PHX) format exporter and importer.

use crate::phx::{PointCloud, PointCloudError, Result, Vertex};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

/// Implements PLY format import and export operations.
pub struct PlyExporter;

impl PlyExporter {
    /// Export to PHX (PLY format)
    pub fn export_phx(
        cloud: &PointCloud,
        binary: bool,
        include_normals: bool,
        include_colors: bool,
    ) -> Result<String> {
        let vertex_count = cloud.vertex_count();
        let mut output = String::new();

        output.push_str("ply\n");
        output.push_str(&format!(
            "format {} 1.0\n",
            if binary {
                "binary_little_endian"
            } else {
                "ascii"
            }
        ));
        output.push_str(&format!("element vertex {}\n", vertex_count));
        output.push_str("property float x\n");
        output.push_str("property float y\n");
        output.push_str("property float z\n");

        if include_normals {
            output.push_str("property float nx\n");
            output.push_str("property float ny\n");
            output.push_str("property float nz\n");
        }

        if include_colors {
            output.push_str("property uchar red\n");
            output.push_str("property uchar green\n");
            output.push_str("property uchar blue\n");
            output.push_str("property uchar alpha\n");
        }

        output.push_str("end_header\n");

        if binary {
            output.push_str("# Binary data follows (not supported in string output)\n");
        } else {
            for vertex in cloud.vertices() {
                let pos = vertex.position;
                output.push_str(&format!("{} {} {}", pos[0], pos[1], pos[2]));

                if include_normals {
                    if let Some(n) = vertex.normal {
                        output.push_str(&format!(" {} {} {}", n[0], n[1], n[2]));
                    } else {
                        output.push_str(" 0 0 0");
                    }
                }

                if include_colors {
                    if let Some(c) = vertex.color {
                        output.push_str(&format!(" {} {} {} {}", c[0], c[1], c[2], c[3]));
                    } else {
                        output.push_str(" 255 255 255 255");
                    }
                }

                output.push('\n');
            }
        }

        Ok(output)
    }

    /// Import from PHX (PLY format)
    pub fn import_phx(data: &str) -> Result<PointCloud> {
        let mut cloud = PointCloud::new("imported");
        let mut lines = data.lines();

        let mut vertex_count = 0;
        let mut has_normals = false;
        let mut has_colors = false;
        let mut in_header = true;

        while let Some(line) = lines.next() {
            let line = line.trim();

            if line == "ply" || line.starts_with("format") {
                continue;
            }

            if line.starts_with("element vertex ") {
                vertex_count = line[15..].parse::<usize>().map_err(|_| {
                    PointCloudError::InvalidFormat("Invalid vertex count".to_string())
                })?;
            }

            if line.starts_with("property float nx") {
                has_normals = true;
            }
            if line.starts_with("property uchar red") {
                has_colors = true;
            }

            if line == "end_header" {
                in_header = false;
                break;
            }
        }

        if in_header {
            return Err(PointCloudError::InvalidFormat(
                "Missing end_header".to_string(),
            ));
        }

        for _ in 0..vertex_count {
            let line = lines.next().ok_or_else(|| {
                PointCloudError::InvalidFormat("Unexpected end of data".to_string())
            })?;

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }

            let x = parts[0].parse::<f32>().unwrap_or(0.0);
            let y = parts[1].parse::<f32>().unwrap_or(0.0);
            let z = parts[2].parse::<f32>().unwrap_or(0.0);

            let mut vertex = Vertex::new(x, y, z);
            let mut idx = 3;

            if has_normals && parts.len() > idx + 2 {
                let nx = parts[idx].parse::<f32>().unwrap_or(0.0);
                let ny = parts[idx + 1].parse::<f32>().unwrap_or(0.0);
                let nz = parts[idx + 2].parse::<f32>().unwrap_or(0.0);
                vertex = vertex.with_normal(nx, ny, nz);
                idx += 3;
            }

            if has_colors && parts.len() > idx + 3 {
                let r = parts[idx].parse::<u8>().unwrap_or(255);
                let g = parts[idx + 1].parse::<u8>().unwrap_or(255);
                let b = parts[idx + 2].parse::<u8>().unwrap_or(255);
                let a = parts[idx + 3].parse::<u8>().unwrap_or(255);
                vertex = vertex.with_color(r, g, b, a);
            }

            cloud.add_vertex(vertex);
        }

        Ok(cloud)
    }

    /// Export PHX to binary bytes
    pub fn export_phx_to_bytes(
        cloud: &PointCloud,
        include_normals: bool,
        include_colors: bool,
    ) -> Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();

        let header = format!(
            "ply\nformat binary_little_endian 1.0\nelement vertex {}\nproperty float x\nproperty float y\nproperty float z\n{}\n{}end_header\n",
            cloud.vertex_count(),
            if include_normals {
                "property float nx\nproperty float ny\nproperty float nz\n"
            } else {
                ""
            },
            if include_colors {
                "property uchar red\nproperty uchar green\nproperty uchar blue\nproperty uchar alpha\n"
            } else {
                ""
            }
        );

        buffer.write_all(header.as_bytes())?;

        for vertex in cloud.vertices() {
            buffer.write_f32::<LittleEndian>(vertex.position[0])?;
            buffer.write_f32::<LittleEndian>(vertex.position[1])?;
            buffer.write_f32::<LittleEndian>(vertex.position[2])?;

            if include_normals {
                let n = vertex.normal.unwrap_or([0.0, 0.0, 0.0]);
                buffer.write_f32::<LittleEndian>(n[0])?;
                buffer.write_f32::<LittleEndian>(n[1])?;
                buffer.write_f32::<LittleEndian>(n[2])?;
            }

            if include_colors {
                let c = vertex.color.unwrap_or([255, 255, 255, 255]);
                buffer.write_u8(c[0])?;
                buffer.write_u8(c[1])?;
                buffer.write_u8(c[2])?;
                buffer.write_u8(c[3])?;
            }
        }

        Ok(buffer)
    }
}
