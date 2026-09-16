//! XYZ ASCII format exporter and importer.

use crate::phx::{PointCloud, Result, Vertex};

/// Implements XYZ format import and export operations.
pub struct XyzExporter;

impl XyzExporter {
    /// Export to XYZ ASCII format
    pub fn export_xyz(
        cloud: &PointCloud,
        include_normals: bool,
        delimiter: &str,
    ) -> Result<String> {
        let mut output = String::new();

        for vertex in cloud.vertices() {
            let pos = vertex.position;
            output.push_str(&format!(
                "{}{}{}{}{}",
                pos[0], delimiter, pos[1], delimiter, pos[2]
            ));

            if include_normals {
                if let Some(n) = vertex.normal {
                    output.push_str(&format!(
                        "{}{}{}{}{}{}",
                        delimiter, n[0], delimiter, n[1], delimiter, n[2]
                    ));
                }
            }

            output.push('\n');
        }

        Ok(output)
    }

    /// Import from XYZ format
    pub fn import_xyz(data: &str, delimiter: &str) -> Result<PointCloud> {
        let mut cloud = PointCloud::new("imported_xyz");

        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let delim = if delimiter == " " {
                " "
            } else if delimiter == "\t" {
                "\t"
            } else {
                delimiter
            };

            let parts: Vec<&str> = line.split(delim).filter(|s| !s.is_empty()).collect();
            if parts.len() < 3 {
                continue;
            }

            let x = parts[0].parse::<f32>().unwrap_or(0.0);
            let y = parts[1].parse::<f32>().unwrap_or(0.0);
            let z = parts[2].parse::<f32>().unwrap_or(0.0);

            let mut vertex = Vertex::new(x, y, z);

            if parts.len() >= 6 {
                let nx = parts[3].parse::<f32>().unwrap_or(0.0);
                let ny = parts[4].parse::<f32>().unwrap_or(0.0);
                let nz = parts[5].parse::<f32>().unwrap_or(0.0);
                vertex = vertex.with_normal(nx, ny, nz);
            }

            cloud.add_vertex(vertex);
        }

        Ok(cloud)
    }
}
