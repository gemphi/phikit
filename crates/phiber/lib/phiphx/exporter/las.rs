//! LAS format exporter.

use crate::phx::{PointCloud, PointCloudError, Result};

/// Implements LAS format export operations.
pub struct LasExporter;

impl LasExporter {
    /// Export to LAS format (simplified)
    pub fn export_las(cloud: &PointCloud, _version: &str) -> Result<String> {
        let mut output = String::new();
        output.push_str("# LAS format requires binary output\n");
        output.push_str(&format!(
            "# {} points would be written\n",
            cloud.vertex_count()
        ));

        Err(PointCloudError::ExportError(
            "LAS export requires binary output - use export_to_bytes()".to_string(),
        ))
    }
}
