//! Mesh data structure.

use crate::phx::point_cloud::PointCloudMetadata;
use crate::phx::vertex::Vertex;

/// Triangle mesh with indexed vertices
/// Supports both triangle lists and triangle strips
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    /// Name/identifier
    pub name: String,
    /// Vertices
    pub vertices: Vec<Vertex>,
    /// Triangle indices (3 per triangle)
    pub indices: Vec<u32>,
    /// Material reference (optional)
    pub material: Option<crate::formats::material::PreviewSurfaceMaterial>,
    /// Material name/path from source file
    pub material_binding: Option<String>,
    /// Transformation matrix from the source asset
    pub transform: Option<glam::Mat4>,
    /// Metadata
    pub metadata: PointCloudMetadata,
}
