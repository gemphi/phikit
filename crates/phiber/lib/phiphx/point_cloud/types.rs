//! Point cloud data structures.

use crate::phx::bounding_box::BoundingBox;
use crate::phx::vertex::Vertex;

/// Point cloud data structure
#[derive(Debug, Clone, PartialEq)]
pub struct PointCloud {
    /// Name/identifier
    pub name: String,
    /// Vertices
    pub(super) vertices: Vec<Vertex>,
    /// Metadata
    pub metadata: PointCloudMetadata,
}

/// Metadata for a point cloud
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct PointCloudMetadata {
    /// Source file format
    pub source_format: Option<String>,
    /// Original file path
    pub source_path: Option<String>,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Additional custom properties
    #[serde(default)]
    pub properties: std::collections::HashMap<String, String>,
}

/// Statistics for a point cloud
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointCloudStats {
    /// Number of vertices
    pub vertex_count: usize,
    /// Bounding box
    pub bounding_box: BoundingBox,
    /// Average point density (points per cubic unit)
    pub average_density: f32,
    /// Average spacing between points
    pub average_spacing: f32,
}
