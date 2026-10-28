//! The Shrinkwrap's target: Blender's `ShrinkwrapTreeData`
//! (`BKE_shrinkwrap_init_tree`) — the target mesh in its own object space,
//! tessellated into corner triangles, with the BVH its nearest-point,
//! nearest-vertex and ray queries walk (`mesh->bvh_corner_tris()` /
//! `bvh_verts()` and their `nearest_callback` / `raycast_callback`).
//!
//! The BVH is a plain median-split tree, not Blender's k-DOP; the queries
//! keep Blender's rules — a node is entered only while it could beat the
//! best so far (`>=` prunes), a primitive replaces the best only when it is
//! strictly closer — so the answer is the same up to exact ties.

use super::math::{closest_on_tri, face_normal, normal_tri, quad_tris, ray_tri_watertight, vertex_normals, RayPrecalc};
use glam::Vec3;

/// `BVHTreeNearest`.
#[derive(Clone, Copy)]
pub(super) struct Nearest {
    pub index: Option<usize>,
    pub dist_sq: f32,
    pub co: Vec3,
    pub no: Vec3,
}

impl Nearest {
    pub(super) fn none() -> Self {
        Self { index: None, dist_sq: f32::MAX, co: Vec3::ZERO, no: Vec3::ZERO }
    }
}

/// `BVHTreeRayHit`.
#[derive(Clone, Copy)]
pub(super) struct Hit {
    pub index: Option<usize>,
    pub dist: f32,
    pub co: Vec3,
    pub no: Vec3,
}

struct Node {
    min: Vec3,
    max: Vec3,
    /// Leaf: `items[start..start + count]`; inner (`count == 0`): children `start`, `start + 1`.
    start: u32,
    count: u32,
}

struct Bvh {
    nodes: Vec<Node>,
    items: Vec<u32>,
}

impl Bvh {
    fn build(boxes: &[(Vec3, Vec3)]) -> Self {
        let mut bvh = Bvh { nodes: Vec::with_capacity(boxes.len() * 2 / 3 + 1), items: (0..boxes.len() as u32).collect() };
        bvh.nodes.push(Node { min: Vec3::ZERO, max: Vec3::ZERO, start: 0, count: 0 });
        let mut todo = vec![(0usize, 0usize, boxes.len())];
        while let Some((node, lo, hi)) = todo.pop() {
            let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
            for &i in &bvh.items[lo..hi] {
                min = min.min(boxes[i as usize].0);
                max = max.max(boxes[i as usize].1);
            }
            // A hair of padding so a primitive on a box face is never lost to rounding.
            let pad = (max - min).max_element().max(min.abs().max(max.abs()).max_element()) * 1e-6;
            let (min, max) = (min - Vec3::splat(pad), max + Vec3::splat(pad));
            if hi - lo <= 4 {
                bvh.nodes[node] = Node { min, max, start: lo as u32, count: (hi - lo) as u32 };
                continue;
            }
            let axis = (max - min).to_array().iter().enumerate().fold(0, |best, (k, e)| match *e > (max - min)[best] {
                true => k,
                false => best,
            });
            let centre = |i: &u32| boxes[*i as usize].0[axis] + boxes[*i as usize].1[axis];
            let mid = (lo + hi) / 2;
            bvh.items[lo..hi].select_nth_unstable_by(mid - lo, |a, b| centre(a).total_cmp(&centre(b)));
            let first = bvh.nodes.len();
            bvh.nodes.push(Node { min: Vec3::ZERO, max: Vec3::ZERO, start: 0, count: 0 });
            bvh.nodes.push(Node { min: Vec3::ZERO, max: Vec3::ZERO, start: 0, count: 0 });
            bvh.nodes[node] = Node { min, max, start: first as u32, count: 0 };
            todo.push((first, lo, mid));
            todo.push((first + 1, mid, hi));
        }
        bvh
    }

    fn box_dist_sq(n: &Node, p: Vec3) -> f32 {
        p.distance_squared(p.clamp(n.min, n.max))
    }

    /// `BLI_bvhtree_find_nearest`: closer child first, pruned at `>=`.
    fn nearest(&self, p: Vec3, best: &mut Nearest, mut leaf: impl FnMut(usize, &mut Nearest)) {
        let mut stack = vec![0u32];
        while let Some(at) = stack.pop() {
            let n = &self.nodes[at as usize];
            if Self::box_dist_sq(n, p) >= best.dist_sq {
                continue;
            }
            match n.count {
                0 => {
                    let (a, b) = (n.start, n.start + 1);
                    let (da, db) = (Self::box_dist_sq(&self.nodes[a as usize], p), Self::box_dist_sq(&self.nodes[b as usize], p));
                    let (near, far) = match da <= db { true => (a, b), false => (b, a) };
                    stack.push(far);
                    stack.push(near);
                }
                count => (n.start..n.start + count).for_each(|k| leaf(self.items[k as usize] as usize, best)),
            }
        }
    }

    /// `BLI_bvhtree_ray_cast`: every node the ray enters before the best hit.
    fn ray(&self, origin: Vec3, dir: Vec3, hit: &mut Hit, mut leaf: impl FnMut(usize, &mut Hit)) {
        let enter = |n: &Node| -> Option<f32> {
            let (mut t0, mut t1) = (f32::MIN, f32::MAX);
            for k in 0..3 {
                match dir[k] == 0.0 {
                    true if origin[k] < n.min[k] || origin[k] > n.max[k] => return None,
                    true => {}
                    false => {
                        let (a, b) = ((n.min[k] - origin[k]) / dir[k], (n.max[k] - origin[k]) / dir[k]);
                        t0 = t0.max(a.min(b));
                        t1 = t1.min(a.max(b));
                    }
                }
            }
            (t0 <= t1 && t1 >= 0.0).then_some(t0)
        };
        let mut stack = vec![0u32];
        while let Some(at) = stack.pop() {
            let n = &self.nodes[at as usize];
            match enter(n) {
                Some(t) if t < hit.dist => {}
                _ => continue,
            }
            match n.count {
                0 => {
                    stack.push(n.start + 1);
                    stack.push(n.start);
                }
                count => (n.start..n.start + count).for_each(|k| leaf(self.items[k as usize] as usize, hit)),
            }
        }
    }
}

/// `ShrinkwrapTreeData`, for one target mesh.
pub(super) struct Target {
    pub positions: Vec<Vec3>,
    /// `corner_tris` as vertex indices, and the face each came from.
    pub tris: Vec<[u32; 3]>,
    pub tri_face: Vec<u32>,
    pub vert_normals: Vec<Vec3>,
    pub face_normals: Vec<Vec3>,
    /// The `sharp_face` attribute, empty when no face is sharp.
    pub sharp: Vec<bool>,
    /// `normals_domain() == Corner`: some faces sharp and some smooth, so a
    /// sharp face's corner normals are its face normal.
    pub corner_domain: bool,
    bvh: Bvh,
    vertex_tree: bool,
}

impl Target {
    /// `BKE_shrinkwrap_init_tree`: a vertex tree for Nearest Vertex, a
    /// triangle tree otherwise; `None` where Blender's returns false.
    pub(super) fn new(positions: Vec<[f32; 3]>, faces: &[Vec<u32>], smooth: &[bool], vertex_tree: bool) -> Option<Self> {
        let positions: Vec<Vec3> = positions.into_iter().map(Vec3::from).collect();
        if positions.is_empty() || (!vertex_tree && faces.is_empty()) {
            return None;
        }
        let face_normals: Vec<Vec3> = faces.iter().map(|f| face_normal(&positions, f)).collect();
        let vert_normals = vertex_normals(&positions, faces.iter().map(|f| f.as_slice()), &face_normals);
        let (mut tris, mut tri_face) = (Vec::new(), Vec::new());
        for (fi, face) in faces.iter().enumerate() {
            let at = |k: usize| positions[face[k] as usize];
            let local: Vec<[usize; 3]> = match face.len() {
                0..=2 => Vec::new(),
                3 => vec![[0, 1, 2]],
                4 => quad_tris([at(0), at(1), at(2), at(3)]).to_vec(),
                _ => crate::scene::polyfill::triangulate(&face.iter().map(|&v| positions[v as usize].to_array()).collect::<Vec<_>>()),
            };
            for t in local {
                tris.push([face[t[0]], face[t[1]], face[t[2]]]);
                tri_face.push(fi as u32);
            }
        }
        let boxes: Vec<(Vec3, Vec3)> = match vertex_tree {
            true => positions.iter().map(|p| (*p, *p)).collect(),
            false => tris
                .iter()
                .map(|t| {
                    let (a, b, c) = (positions[t[0] as usize], positions[t[1] as usize], positions[t[2] as usize]);
                    (a.min(b).min(c), a.max(b).max(c))
                })
                .collect(),
        };
        if boxes.is_empty() {
            return None;
        }
        let sharp: Vec<bool> = smooth.iter().map(|s| !s).collect();
        let any_sharp = sharp.iter().any(|s| *s);
        let corner_domain = any_sharp && sharp.iter().any(|s| !s);
        let sharp = match any_sharp { true => sharp, false => Vec::new() };
        Some(Self { positions, tris, tri_face, vert_normals, face_normals, sharp, corner_domain, bvh: Bvh::build(&boxes), vertex_tree })
    }

    pub(super) fn tri(&self, i: usize) -> [Vec3; 3] {
        let t = self.tris[i];
        [self.positions[t[0] as usize], self.positions[t[1] as usize], self.positions[t[2] as usize]]
    }

    /// `BLI_bvhtree_find_nearest` with `mesh_corner_tris_nearest_point`, or
    /// over the vertex tree (no callback: the point itself) for Nearest Vertex.
    pub(super) fn nearest(&self, co: Vec3, nearest: &mut Nearest) {
        match self.vertex_tree {
            true => self.bvh.nearest(co, nearest, |v, best| {
                let p = self.positions[v];
                let d = co.distance_squared(p);
                if d < best.dist_sq {
                    *best = Nearest { index: Some(v), dist_sq: d, co: p, no: best.no };
                }
            }),
            false => self.bvh.nearest(co, nearest, |i, best| {
                let [t0, t1, t2] = self.tri(i);
                let p = closest_on_tri(co, t0, t1, t2);
                let d = co.distance_squared(p);
                if d < best.dist_sq {
                    *best = Nearest { index: Some(i), dist_sq: d, co: p, no: normal_tri(t0, t1, t2) };
                }
            }),
        }
    }

    /// `BLI_bvhtree_ray_cast` with `mesh_corner_tris_spherecast` at radius 0
    /// (`bvhtree_ray_tri_intersection`, watertight).
    pub(super) fn ray_cast(&self, origin: Vec3, dir: Vec3, hit: &mut Hit) {
        let pre = RayPrecalc::new(dir);
        self.bvh.ray(origin, dir, hit, |i, best| {
            let [t0, t1, t2] = self.tri(i);
            let dist = ray_tri_watertight(origin, &pre, t0, t1, t2).unwrap_or(f32::MAX);
            if dist >= 0.0 && dist < best.dist {
                *best = Hit { index: Some(i), dist, co: origin + dir * dist, no: normal_tri(t0, t1, t2) };
            }
        });
    }
}
