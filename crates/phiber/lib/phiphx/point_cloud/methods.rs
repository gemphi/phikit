//! Point cloud methods.

use crate::phx::bounding_box::BoundingBox;
use crate::phx::error::{PointCloudError, Result};
use crate::phx::exporter::PointCloudExporter;
use crate::phx::formats::PointCloudFormat;
use crate::phx::vertex::Vertex;

use super::types::{PointCloud, PointCloudMetadata, PointCloudStats};

impl PointCloud {
    /// Create a new empty point cloud
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vertices: Vec::new(),
            metadata: PointCloudMetadata::default(),
        }
    }

    /// Create with capacity hint
    pub fn with_capacity(name: impl Into<String>, capacity: usize) -> Self {
        Self {
            name: name.into(),
            vertices: Vec::with_capacity(capacity),
            metadata: PointCloudMetadata::default(),
        }
    }

    /// Add a vertex
    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.vertices.push(vertex);
    }

    /// Add multiple vertices
    pub fn add_vertices(&mut self, vertices: impl IntoIterator<Item = Vertex>) {
        self.vertices.extend(vertices);
    }

    /// Get all vertices
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    /// Get mutable vertices
    pub fn vertices_mut(&mut self) -> &mut Vec<Vertex> {
        &mut self.vertices
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Calculate bounding box
    pub fn calculate_bounds(&self) -> BoundingBox {
        BoundingBox::from_vertices(&self.vertices).unwrap_or_default()
    }

    /// Calculate statistics
    pub fn statistics(&self) -> PointCloudStats {
        let bbox = self.calculate_bounds();
        let volume = bbox.size()[0] * bbox.size()[1] * bbox.size()[2];
        let density = if volume > 0.0 {
            self.vertex_count() as f32 / volume
        } else {
            0.0
        };

        // Estimate average spacing
        let avg_spacing = if self.vertex_count() > 1 {
            bbox.diagonal() / (self.vertex_count() as f32).cbrt()
        } else {
            0.0
        };

        PointCloudStats {
            vertex_count: self.vertex_count(),
            bounding_box: bbox,
            average_density: density,
            average_spacing: avg_spacing,
        }
    }

    /// Transform all vertices
    pub fn transform(&mut self, matrix: &[f32; 16]) {
        for v in &mut self.vertices {
            *v = v.transform(matrix);
        }
    }

    /// Downsample using voxel grid filter
    pub fn downsample(&self, voxel_size: f32) -> Self {
        if voxel_size <= 0.0 || self.is_empty() {
            return self.clone();
        }

        let bbox = self.calculate_bounds();
        let size = bbox.size();

        // Calculate grid dimensions
        let nx = (size[0] / voxel_size).ceil() as usize + 1;
        let ny = (size[1] / voxel_size).ceil() as usize + 1;
        let nz = (size[2] / voxel_size).ceil() as usize + 1;

        // Use a hash set to track occupied voxels
        let mut occupied: std::collections::HashSet<(usize, usize, usize)> =
            std::collections::HashSet::new();
        let mut result = PointCloud::with_capacity(&self.name, self.vertex_count() / 10);
        result.metadata = self.metadata.clone();

        for v in &self.vertices {
            let ix = ((v.position[0] - bbox.min[0]) / voxel_size) as usize;
            let iy = ((v.position[1] - bbox.min[1]) / voxel_size) as usize;
            let iz = ((v.position[2] - bbox.min[2]) / voxel_size) as usize;

            let key = (ix.min(nx), iy.min(ny), iz.min(nz));
            if occupied.insert(key) {
                result.add_vertex(*v);
            }
        }

        result
    }

    /// Filter vertices by bounding box
    pub fn filter_by_bounds(&self, bbox: &BoundingBox) -> Self {
        let mut result = PointCloud::with_capacity(&self.name, self.vertex_count());
        result.metadata = self.metadata.clone();

        for v in &self.vertices {
            if bbox.contains(&v.position) {
                result.add_vertex(*v);
            }
        }

        result
    }

    /// Merge with another point cloud
    pub fn merge(&self, other: &PointCloud) -> Self {
        let mut result = PointCloud::with_capacity(
            format!("{}_{}", self.name, other.name),
            self.vertex_count() + other.vertex_count(),
        );
        result.metadata = self.metadata.clone();

        for v in self.vertices() {
            result.add_vertex(*v);
        }
        for v in other.vertices() {
            result.add_vertex(*v);
        }

        result
    }

    /// Export to a specific format
    pub fn export(&self, format: &PointCloudFormat) -> Result<String> {
        PointCloudExporter::export_point_cloud(self, format)
    }

    /// Import from a specific format
    pub fn import(data: &str, format: &PointCloudFormat) -> Result<Self> {
        PointCloudExporter::import_point_cloud(data, format)
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PointCloudError::Serialization(e.to_string()))
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| PointCloudError::Serialization(e.to_string()))
    }

    /// Clear all vertices
    pub fn clear(&mut self) {
        self.vertices.clear();
    }
}
