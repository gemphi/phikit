//! PCD format exporter.

use crate::phx::{PointCloud, Result};

/// Implements PCD format export operations.
pub struct PcdExporter;

impl PcdExporter {
    /// Export to PCD format
    pub fn export_pcd(cloud: &PointCloud, binary: bool, _compressed: bool) -> Result<String> {
        let mut output = String::new();
        let vertex_count = cloud.vertex_count();

        output.push_str("# .PCD v0.7 - Point Cloud Data file format\n");
        output.push_str("VERSION 0.7\n");
        output.push_str("FIELDS x y z\n");
        output.push_str("SIZE 4 4 4\n");
        output.push_str("TYPE F F F\n");
        output.push_str("COUNT 1 1 1\n");
        output.push_str(&format!("WIDTH {}\n", vertex_count));
        output.push_str("HEIGHT 1\n");
        output.push_str(&format!("POINTS {}\n", vertex_count));
        output.push_str(&format!(
            "DATA {}\n",
            if binary { "binary" } else { "ascii" }
        ));

        if binary {
            output.push_str("# Binary data follows (returning ASCII for now)\n");
        }

        for vertex in cloud.vertices() {
            let pos = vertex.position;
            output.push_str(&format!("{} {} {}\n", pos[0], pos[1], pos[2]));
        }

        Ok(output)
    }
}
