//! Triangle mesh representation and sampling methods.

use super::bvh::{BvhNode, SimpleRng};
use super::config::{MeshToCloudConfig, SamplingMethod};
use super::selector::MeshSource;
use crate::phx::{PointCloud, Vertex};

/// A point sampled from a mesh triangle
#[derive(Debug, Clone, Copy)]
pub struct SampledPoint {
    pub position: [f32; 3],
    pub normal: Option<[f32; 3]>,
    pub color: Option<[f32; 4]>,
}

/// Triangle mesh representation for conversion
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    /// Name of the mesh (e.g. from USD prim)
    pub name: String,
    /// Vertex positions
    pub positions: Vec<[f32; 3]>,
    /// Normals (optional)
    pub normals: Option<Vec<[f32; 3]>>,
    /// Colors (optional, RGBA)
    pub colors: Option<Vec<[f32; 4]>>,
    /// UV coordinates (optional)
    pub uvs: Option<Vec<[f32; 2]>>,
    /// Triangle indices (3 indices per triangle)
    pub indices: Vec<[u32; 3]>,
    /// Transformation matrix from the source asset (e.g., from USD xform)
    pub transform: Option<glam::Mat4>,
    /// BVH acceleration structure (built on demand)
    pub bvh: Option<BvhNode>,
}

impl Mesh {
    /// Create empty mesh
    pub fn new() -> Self {
        Self::default()
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Get triangle count
    pub fn triangle_count(&self) -> usize {
        self.indices.len()
    }

    /// Build BVH acceleration structure
    pub fn build_bvh(&mut self, max_depth: usize) {
        self.bvh = BvhNode::build_from_mesh(self, max_depth);
    }

    /// Calculate triangle area
    pub fn triangle_area(&self, tri_idx: usize) -> f32 {
        if tri_idx >= self.indices.len() {
            return 0.0;
        }

        let tri = self.indices[tri_idx];
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        if i0 >= self.positions.len() || i1 >= self.positions.len() || i2 >= self.positions.len() {
            return 0.0;
        }

        let p0 = self.positions[i0];
        let p1 = self.positions[i1];
        let p2 = self.positions[i2];

        let e0 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e1 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

        let cross = [
            e0[1] * e1[2] - e0[2] * e1[1],
            e0[2] * e1[0] - e0[0] * e1[2],
            e0[0] * e1[1] - e0[1] * e1[0],
        ];

        let cross_len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        cross_len * 0.5
    }

    /// Calculate total surface area
    pub fn surface_area(&self) -> f32 {
        (0..self.triangle_count())
            .map(|i| self.triangle_area(i))
            .sum()
    }

    /// Get triangle normal
    pub fn triangle_normal(&self, tri_idx: usize) -> Option<[f32; 3]> {
        if tri_idx >= self.indices.len() {
            return None;
        }

        let tri = self.indices[tri_idx];
        let p0 = self.positions.get(tri[0] as usize)?;
        let p1 = self.positions.get(tri[1] as usize)?;
        let p2 = self.positions.get(tri[2] as usize)?;

        let e0 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e1 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

        let cross = [
            e0[1] * e1[2] - e0[2] * e1[1],
            e0[2] * e1[0] - e0[0] * e1[2],
            e0[0] * e1[1] - e0[1] * e1[0],
        ];

        let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        if len > 0.0 {
            Some([cross[0] / len, cross[1] / len, cross[2] / len])
        } else {
            None
        }
    }

    /// Sample a random point on a triangle
    pub fn sample_triangle(&self, tri_idx: usize, u: f32, v: f32) -> Option<SampledPoint> {
        if tri_idx >= self.indices.len() {
            return None;
        }

        let (u, v) = if u + v > 1.0 {
            (1.0 - u, 1.0 - v)
        } else {
            (u, v)
        };
        let w = 1.0 - u - v;

        let tri = self.indices[tri_idx];
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let p0 = self.positions[i0];
        let p1 = self.positions[i1];
        let p2 = self.positions[i2];

        let position = [
            p0[0] * w + p1[0] * u + p2[0] * v,
            p0[1] * w + p1[1] * u + p2[2] * v,
            p0[2] * w + p1[2] * u + p2[2] * v,
        ];

        let normal = if let Some(ref normals) = self.normals {
            let n0 = normals.get(i0)?;
            let n1 = normals.get(i1)?;
            let n2 = normals.get(i2)?;

            let nx = n0[0] * w + n1[0] * u + n2[0] * v;
            let ny = n0[1] * w + n1[1] * u + n2[1] * v;
            let nz = n0[2] * w + n1[2] * u + n2[2] * v;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();

            if len > 0.0 {
                Some([nx / len, ny / len, nz / len])
            } else {
                None
            }
        } else {
            self.triangle_normal(tri_idx)
        };

        let color = self.colors.as_ref().and_then(|colors| {
            let c0 = colors.get(i0)?;
            let c1 = colors.get(i1)?;
            let c2 = colors.get(i2)?;

            Some([
                (c0[0] * w + c1[0] * u + c2[0] * v).clamp(0.0, 1.0),
                (c0[1] * w + c1[1] * u + c2[1] * v).clamp(0.0, 1.0),
                (c0[2] * w + c1[2] * u + c2[2] * v).clamp(0.0, 1.0),
                (c0[3] * w + c1[3] * u + c2[3] * v).clamp(0.0, 1.0),
            ])
        });

        Some(SampledPoint {
            position,
            normal,
            color,
        })
    }

    /// Convert mesh to point cloud
    pub fn to_point_cloud(&self, config: &MeshToCloudConfig) -> PointCloud {
        let mut cloud = PointCloud::new("mesh_points");

        if self.triangle_count() == 0 || self.vertex_count() == 0 {
            return cloud;
        }

        let total_area = self.surface_area();
        let target_count =
            ((total_area * config.density) as usize).clamp(config.min_points, config.max_points);

        if total_area <= 0.0 || target_count == 0 {
            return cloud;
        }

        let mut rng = SimpleRng::new(config.seed);
        let use_bvh =
            config.use_bvh || matches!(config.sampling_method, SamplingMethod::BvhStratified);

        let bvh = if use_bvh {
            BvhNode::build_from_mesh(
                self,
                if config.bvh_depth == 0 {
                    16
                } else {
                    config.bvh_depth
                },
            )
        } else {
            None
        };

        let cumulative: Vec<f32> = if !use_bvh {
            let triangle_areas: Vec<f32> = (0..self.triangle_count())
                .map(|i| self.triangle_area(i))
                .collect();
            triangle_areas
                .iter()
                .scan(0.0, |acc, &area| {
                    *acc += area / total_area;
                    Some(*acc)
                })
                .collect()
        } else {
            Vec::new()
        };

        for _ in 0..target_count {
            let tri_idx = match config.sampling_method {
                SamplingMethod::Uniform | SamplingMethod::AreaWeighted
                    if config.uniform_sampling =>
                {
                    (rng.next_f32() * self.triangle_count() as f32) as usize % self.triangle_count()
                }
                SamplingMethod::Uniform | SamplingMethod::AreaWeighted => {
                    let r = rng.next_f32();
                    cumulative
                        .iter()
                        .position(|&c| c >= r)
                        .unwrap_or(self.triangle_count() - 1)
                }
                SamplingMethod::BvhStratified => {
                    if let Some(ref bvh_node) = bvh {
                        bvh_node.sample_triangle_bvh(&mut rng, self).unwrap_or(0)
                    } else {
                        (rng.next_f32() * self.triangle_count() as f32) as usize
                            % self.triangle_count()
                    }
                }
            };

            let u = rng.next_f32();
            let v = rng.next_f32();

            if let Some(point) = self.sample_triangle(tri_idx, u, v) {
                let mut vertex =
                    Vertex::new(point.position[0], point.position[1], point.position[2]);

                if config.include_normals {
                    if let Some(n) = point.normal {
                        vertex = vertex.with_normal(n[0], n[1], n[2]);
                    }
                }

                if config.include_colors {
                    if let Some(c) = point.color {
                        vertex = vertex.with_color(
                            (c[0] * 255.0) as u8,
                            (c[1] * 255.0) as u8,
                            (c[2] * 255.0) as u8,
                            (c[3] * 255.0) as u8,
                        );
                    }
                }

                cloud.add_vertex(vertex);
            }
        }

        cloud
    }
}

impl MeshSource for Mesh {
    fn name(&self) -> &str {
        &self.name
    }
    fn vertices(&self) -> &[[f32; 3]] {
        &self.positions
    }
    fn indices(&self) -> &[[u32; 3]] {
        &self.indices
    }
    fn matrix_world(&self) -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}
