//! Axis-aligned bounding box.

use crate::phx::vertex::Vertex;

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoundingBox {
    /// Minimum corner [x, y, z]
    pub min: [f32; 3],
    /// Maximum corner [x, y, z]
    pub max: [f32; 3],
}

impl BoundingBox {
    /// Create a new bounding box
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    /// Get the center point
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    /// Get the size [width, height, depth]
    pub fn size(&self) -> [f32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    /// Get diagonal length
    pub fn diagonal(&self) -> f32 {
        let size = self.size();
        (size[0] * size[0] + size[1] * size[1] + size[2] * size[2]).sqrt()
    }

    /// Check if a point is inside the bounding box
    pub fn contains(&self, point: &[f32; 3]) -> bool {
        point[0] >= self.min[0]
            && point[0] <= self.max[0]
            && point[1] >= self.min[1]
            && point[1] <= self.max[1]
            && point[2] >= self.min[2]
            && point[2] <= self.max[2]
    }

    /// Expand to include a point
    pub fn expand(&mut self, point: &[f32; 3]) {
        for i in 0..3 {
            self.min[i] = self.min[i].min(point[i]);
            self.max[i] = self.max[i].max(point[i]);
        }
    }

    /// Create from a list of vertices
    pub fn from_vertices(vertices: &[Vertex]) -> Option<Self> {
        if vertices.is_empty() {
            return None;
        }

        let mut bbox = Self {
            min: vertices[0].position,
            max: vertices[0].position,
        };

        for v in &vertices[1..] {
            bbox.expand(&v.position);
        }

        Some(bbox)
    }
}
