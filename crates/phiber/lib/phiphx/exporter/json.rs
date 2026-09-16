//! JSON format exporter and importer.

use crate::phx::{PointCloud, PointCloudError, Result};

/// Implements JSON format import and export operations.
pub struct JsonExporter;

impl JsonExporter {
    /// Export to JSON format
    pub fn export_json(cloud: &PointCloud, pretty: bool) -> Result<String> {
        if pretty {
            cloud.to_json()
        } else {
            serde_json::to_string(cloud).map_err(|e| PointCloudError::Serialization(e.to_string()))
        }
    }

    /// Import from JSON format
    pub fn import_json(data: &str) -> Result<PointCloud> {
        PointCloud::from_json(data)
    }
}
