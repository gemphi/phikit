//! BVH spatial acceleration for point cloud sampling.

use super::mesh::Mesh;

/// Bounding box for BVH nodes
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl BoundingBox {
    pub fn new() -> Self {
        Self {
            min: [f32::INFINITY, f32::INFINITY, f32::INFINITY],
            max: [f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY],
        }
    }

    pub fn expand(&mut self, p: [f32; 3]) {
        for i in 0..3 {
            self.min[i] = self.min[i].min(p[i]);
            self.max[i] = self.max[i].max(p[i]);
        }
    }

    pub fn surface_area(&self) -> f32 {
        let dx = self.max[0] - self.min[0];
        let dy = self.max[1] - self.min[1];
        let dz = self.max[2] - self.min[2];
        if dx < 0.0 || dy < 0.0 || dz < 0.0 {
            0.0
        } else {
            2.0 * (dx * dy + dy * dz + dz * dx)
        }
    }
}

/// BVH node for spatial acceleration
#[derive(Debug, Clone)]
pub enum BvhNode {
    /// Leaf node containing triangle indices
    Leaf {
        bbox: BoundingBox,
        triangles: Vec<usize>,
    },
    /// Internal node with children
    Internal {
        bbox: BoundingBox,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
    },
}

impl BvhNode {
    /// Build BVH from mesh triangles
    pub fn build_from_mesh(mesh: &Mesh, max_depth: usize) -> Option<Self> {
        if mesh.indices.is_empty() {
            return None;
        }

        let triangle_indices: Vec<usize> = (0..mesh.indices.len()).collect();
        Self::build_recursive(mesh, &triangle_indices, max_depth, 0)
    }

    fn build_recursive(
        mesh: &Mesh,
        triangles: &[usize],
        max_depth: usize,
        current_depth: usize,
    ) -> Option<Self> {
        if triangles.is_empty() {
            return None;
        }

        let mut bbox = BoundingBox::new();
        for &tri_idx in triangles {
            let tri = mesh.indices[tri_idx];
            for i in 0..3 {
                let idx = tri[i] as usize;
                if idx < mesh.positions.len() {
                    bbox.expand(mesh.positions[idx]);
                }
            }
        }

        if triangles.len() <= 4 || current_depth >= max_depth {
            return Some(BvhNode::Leaf {
                bbox,
                triangles: triangles.to_vec(),
            });
        }

        let extent = [
            bbox.max[0] - bbox.min[0],
            bbox.max[1] - bbox.min[1],
            bbox.max[2] - bbox.min[2],
        ];
        let axis = if extent[0] > extent[1] && extent[0] > extent[2] {
            0
        } else if extent[1] > extent[2] {
            1
        } else {
            2
        };

        let mut sorted_triangles: Vec<(usize, f32)> = triangles
            .iter()
            .map(|&tri_idx| {
                let tri = mesh.indices[tri_idx];
                let mut centroid = [0.0f32; 3];
                for i in 0..3 {
                    let idx = tri[i] as usize;
                    if idx < mesh.positions.len() {
                        for j in 0..3 {
                            centroid[j] += mesh.positions[idx][j];
                        }
                    }
                }
                for j in 0..3 {
                    centroid[j] /= 3.0;
                }
                (tri_idx, centroid[axis])
            })
            .collect();

        sorted_triangles.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let mid = sorted_triangles.len() / 2;
        let left_triangles: Vec<usize> = sorted_triangles[..mid].iter().map(|x| x.0).collect();
        let right_triangles: Vec<usize> = sorted_triangles[mid..].iter().map(|x| x.0).collect();

        let left = Self::build_recursive(mesh, &left_triangles, max_depth, current_depth + 1);
        let right = Self::build_recursive(mesh, &right_triangles, max_depth, current_depth + 1);

        match (left, right) {
            (Some(l), Some(r)) => Some(BvhNode::Internal {
                bbox,
                left: Box::new(l),
                right: Box::new(r),
            }),
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => Some(BvhNode::Leaf {
                bbox,
                triangles: triangles.to_vec(),
            }),
        }
    }

    /// Sample a random triangle using BVH for stratified sampling
    pub fn sample_triangle_bvh(&self, rng: &mut SimpleRng, mesh: &Mesh) -> Option<usize> {
        match self {
            BvhNode::Leaf { triangles, .. } => {
                if triangles.is_empty() {
                    None
                } else {
                    let idx = (rng.next_f32() * triangles.len() as f32) as usize % triangles.len();
                    Some(triangles[idx])
                }
            }
            BvhNode::Internal { left, right, .. } => {
                let left_sa = left.bounding_box().surface_area();
                let right_sa = right.bounding_box().surface_area();
                let total_sa = left_sa + right_sa;

                if total_sa <= 0.0 {
                    if rng.next_f32() < 0.5 {
                        left.sample_triangle_bvh(rng, mesh)
                    } else {
                        right.sample_triangle_bvh(rng, mesh)
                    }
                } else {
                    let left_prob = left_sa / total_sa;
                    if rng.next_f32() < left_prob {
                        left.sample_triangle_bvh(rng, mesh)
                    } else {
                        right.sample_triangle_bvh(rng, mesh)
                    }
                }
            }
        }
    }

    fn bounding_box(&self) -> &BoundingBox {
        match self {
            BvhNode::Leaf { bbox, .. } => bbox,
            BvhNode::Internal { bbox, .. } => bbox,
        }
    }
}

/// Simple pseudo-random number generator
#[derive(Debug, Clone, Copy)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        let state = if seed == 0 { 0x123456789ABCDEF0 } else { seed };
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545F4914F6CDD1D)
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 32) as f32 / u32::MAX as f32
    }
}
