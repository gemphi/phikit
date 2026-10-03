//! A pick tree per mesh: phiray's BVH (`Bvh::build_compact`, the builder
//! the tracer uses, optimised) over the mesh's triangles in the mesh's own
//! space, built once per mesh and kept by its address, so a click walks a
//! few dozen nodes rather than every triangle.
//!
//! Before, a click took every vertex of every object into world space and
//! tested every triangle of each object whose box the ray crossed: on a
//! fifteen-million-triangle scene that was seconds, in a debug build, for
//! one click. The tree is in object space, so a moved or posed object
//! needs no new tree: the ray goes into object space instead
//! (`world = M · local`, so a ray `o + t·d` in the world is
//! `M⁻¹o + t·M⁻¹d` there, with the same `t`).

use super::BPickRay;
use super::triangle::{hits_bounds, triangle_t};
use crate::phiphx::scene::SceneMesh;
use crate::v2::scene::keep::BMeshKeep;
use glam::{Mat4, Vec3};
use phiray::{Aabb, Bvh};
use std::sync::Arc;

/// Triangles per leaf — the tracer's own setting.
const LEAF: u32 = 4;
/// A mesh with fewer triangles than this is walked whole: a tree would cost
/// more than it saves.
pub const WHOLE: usize = 64;

/// The tree over one mesh's triangles.
pub struct BPickTree {
    bvh: Bvh,
    mesh: Arc<SceneMesh>,
}

static KEPT: BMeshKeep<BPickTree> = BMeshKeep::new();

impl BPickTree {
    /// The tree of `mesh`, built once and kept while the mesh lives — here
    /// on first use, or as the file loads (`scene/warm.rs`).
    pub fn of(mesh: &Arc<SceneMesh>) -> Arc<Self> {
        KEPT.of(mesh, Self::build)
    }

    /// Whether `mesh` has its tree already.
    pub fn ready(mesh: &Arc<SceneMesh>) -> bool {
        KEPT.has(mesh)
    }

    fn build(mesh: &Arc<SceneMesh>) -> Self {
        let v = &mesh.vertices;
        let prims: Vec<Aabb> = mesh
            .indices
            .chunks_exact(3)
            .map(|t| {
                let p = |i: u32| v.get(i as usize).copied().unwrap_or([0.0; 3]);
                let (a, b, c) = (Vec3::from(p(t[0])), Vec3::from(p(t[1])), Vec3::from(p(t[2])));
                Aabb::new(a.min(b).min(c).to_array(), a.max(b).max(c).to_array())
            })
            .collect();
        Self { bvh: Bvh::build_compact(&prims, LEAF), mesh: Arc::clone(mesh) }
    }

    /// The mesh's box in the world, placed by `matrix_world`: its own box's
    /// eight corners through the matrix.
    pub fn world_bounds(&self, matrix_world: &[[f32; 4]; 4]) -> (Vec3, Vec3) {
        let m = Mat4::from_cols_array_2d(matrix_world);
        let Some(root) = self.bvh.nodes.first() else { return (Vec3::ZERO, Vec3::ZERO) };
        let b = root.bounds();
        let (lo, hi) = (Vec3::from(b.lo), Vec3::from(b.hi));
        (0..8).map(|k| m.transform_point3(Vec3::new(if k & 1 == 0 { lo.x } else { hi.x }, if k & 2 == 0 { lo.y } else { hi.y }, if k & 4 == 0 { lo.z } else { hi.z }))).fold((Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)), |(a, b), p| (a.min(p), b.max(p)))
    }

    /// The nearest triangle of the mesh, placed by `matrix_world`, along
    /// `ray` — the distance along the ray.
    pub fn nearest(&self, matrix_world: &[[f32; 4]; 4], ray: BPickRay) -> Option<f32> {
        let inverse = Mat4::from_cols_array_2d(matrix_world).inverse();
        let local = BPickRay { origin: inverse.transform_point3(ray.origin), dir: inverse.transform_vector3(ray.dir) };
        let (v, idx, nodes) = (&self.mesh.vertices, &self.mesh.indices, &self.bvh.nodes);
        let Some(root) = nodes.first() else { return None };
        let bounds = root.bounds();
        if !hits_bounds(Vec3::from(bounds.lo), Vec3::from(bounds.hi), local) {
            return None;
        }
        let mut best = f32::INFINITY;
        let mut stack = vec![0usize];
        while let Some(i) = stack.pop() {
            let node = &nodes[i];
            let b = node.bounds();
            if !hits_bounds(Vec3::from(b.lo), Vec3::from(b.hi), local) {
                continue;
            }
            if node.is_leaf() {
                for slot in node.lo.index()..node.hi.i {
                    let t = self.bvh.order[slot as usize] as usize * 3;
                    let p = |k: usize| v.get(idx[t + k] as usize).map(|p| Vec3::from(*p));
                    if let (Some(a), Some(b), Some(c)) = (p(0), p(1), p(2))
                        && let Some(d) = triangle_t(a, b, c, local)
                    {
                        best = best.min(d);
                    }
                }
            } else {
                stack.push(node.lo.i as usize);
                stack.push(node.hi.i as usize);
            }
        }
        best.is_finite().then_some(best)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::scene::SceneNormals;

    fn grid(n: usize) -> Arc<SceneMesh> {
        // An n × n sheet of quads in the XY plane at z = 0.
        let mut v = Vec::new();
        for y in 0..=n {
            for x in 0..=n {
                v.push([x as f32, y as f32, 0.0]);
            }
        }
        let mut i = Vec::new();
        let row = (n + 1) as u32;
        for y in 0..n as u32 {
            for x in 0..n as u32 {
                let p = y * row + x;
                i.extend([p, p + 1, p + row, p + 1, p + row + 1, p + row]);
            }
        }
        Arc::new(SceneMesh { vertices: v, indices: i, normals: SceneNormals::PerFace(Vec::new()), corners: Default::default() })
    }

    #[test]
    fn the_tree_finds_what_the_whole_walk_finds_through_a_placed_mesh() {
        let mesh = grid(40);
        let tree = BPickTree::of(&mesh);
        assert!(Arc::ptr_eq(&tree, &BPickTree::of(&mesh)), "built once");
        // Moved to (10, 0, 5), scaled by 2: the sheet at z = 5 over x 10..90.
        let m = [[2.0, 0.0, 0.0, 0.0], [0.0, 2.0, 0.0, 0.0], [0.0, 0.0, 2.0, 0.0], [10.0, 0.0, 5.0, 1.0]];
        let down = |x: f32, y: f32| BPickRay { origin: Vec3::new(x, y, 20.0), dir: Vec3::NEG_Z };
        let hit = tree.nearest(&m, down(33.3, 21.7)).expect("on the sheet");
        assert!((hit - 15.0).abs() < 1e-4, "{hit}");
        assert!(tree.nearest(&m, down(95.0, 21.7)).is_none(), "past its edge");
        assert!(tree.nearest(&m, BPickRay { origin: Vec3::new(33.3, 21.7, 20.0), dir: Vec3::Z }).is_none(), "behind the ray");
        // The same as the whole walk.
        let world: Vec<Vec3> = mesh.vertices.iter().map(|v| Mat4::from_cols_array_2d(&m).transform_point3(Vec3::from(*v))).collect();
        let whole = super::super::nearest_triangle(&world, &mesh.indices, down(33.3, 21.7)).unwrap();
        assert!((hit - whole).abs() < 1e-4);
    }
}
