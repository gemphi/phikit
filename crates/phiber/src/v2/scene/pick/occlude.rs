//! What a box can see — `view3d.select_box` with X-ray off reads the depth
//! buffer under the box (`do_object_box_select` → `view3d_opengl_select`),
//! so an object hidden behind another is not taken. Here the box is
//! sampled with rays, each giving the nearest object along it; an object
//! whose projected bounds overlap the box counts only when some ray sees
//! it. The grid is as fine as a triangle budget allows (up to 32 × 32), and
//! every candidate whose origin is inside the box gets one ray of its own,
//! so a small object in a big box is not missed between samples.
//!
//! The scene is moved into world space once ([`BPickScene`]) — a pick
//! rebuilds each object's vertices per ray, which a few hundred rays
//! cannot afford.

use super::triangle::{bounds, hits_bounds};
use super::{BPickRay, BPickView, nearest_triangle};
use crate::v2::scene::boxsel::BBox2;
use crate::v2::scene::{BObject3D, MESH_SCALE};
use crate::phiphx::spatial::SpatialObject3D;
use glam::{Mat4, Vec3};
use std::borrow::Cow;
use std::collections::HashSet;

/// Rays × triangles one box select may spend on the samples.
const BUDGET: usize = 40_000_000;
/// The finest grid, per side.
const MAX_SIDE: usize = 32;

/// One object with its world box: a small one moved into world space, a
/// large one through its tree in its own space (`tree.rs`).
struct BWorldObject<'a> {
    name: &'a str,
    body: BBody<'a>,
    lo: Vec3,
    hi: Vec3,
}

enum BBody<'a> {
    Whole { world: Vec<Vec3>, indices: Cow<'a, [u32]> },
    Tree { tree: std::sync::Arc<super::tree::BPickTree>, matrix: [[f32; 4]; 4] },
}

impl BBody<'_> {
    /// How many triangles a ray costs: the tree's a few dozen tests.
    fn cost(&self) -> usize {
        match self {
            Self::Whole { indices, .. } => indices.len() / 3,
            Self::Tree { .. } => super::tree::WHOLE,
        }
    }
}

/// The scene, moved into world space once for many rays.
pub struct BPickScene<'a> {
    objects: Vec<BWorldObject<'a>>,
}

impl<'a> BPickScene<'a> {
    /// The objects `allowed` — one whose arrow toggle is set is left out,
    /// so it hides nothing behind it, as it is left out of Blender's
    /// selection pass.
    pub fn new(scene: &'a [BObject3D], allowed: impl Fn(&str) -> bool) -> Self {
        let objects = scene
            .iter()
            .filter(|object| allowed(object.name()))
            .map(|object| {
                let (world, indices): (Vec<Vec3>, Cow<'a, [u32]>) = match object {
                    BObject3D::Primitive { shape, x, y, .. } => {
                        let (v, idx, _) = SpatialObject3D::buffers(shape.mesh_type());
                        let (ox, oy) = (x / MESH_SCALE, y / MESH_SCALE);
                        (v.iter().map(|v| Vec3::new(ox + v.0, oy + v.2, v.1)).collect(), Cow::Owned(idx))
                    }
                    BObject3D::Loaded { mesh, matrix_world, .. } if mesh.indices.len() / 3 >= super::tree::WHOLE => {
                        let tree = super::tree::BPickTree::of(mesh);
                        let (lo, hi) = tree.world_bounds(matrix_world);
                        return BWorldObject { name: object.name(), body: BBody::Tree { tree, matrix: *matrix_world }, lo, hi };
                    }
                    BObject3D::Loaded { mesh, matrix_world, .. } => {
                        let m = Mat4::from_cols_array_2d(matrix_world);
                        (mesh.vertices.iter().map(|v| m.transform_point3(Vec3::from(*v))).collect(), Cow::Borrowed(&mesh.indices[..]))
                    }
                };
                let (lo, hi) = bounds(&world);
                BWorldObject { name: object.name(), body: BBody::Whole { world, indices }, lo, hi }
            })
            .collect();
        Self { objects }
    }

    fn triangles(&self) -> usize {
        self.objects.iter().map(|o| o.body.cost()).sum()
    }

    /// The nearest object along `ray`.
    pub fn nearest(&self, ray: BPickRay) -> Option<&'a str> {
        let mut best: Option<(&'a str, f32)> = None;
        for o in &self.objects {
            if !hits_bounds(o.lo, o.hi, ray) {
                continue;
            }
            let hit = match &o.body {
                BBody::Whole { world, .. } if world.is_empty() => None,
                BBody::Whole { world, indices } => nearest_triangle(world, indices, ray),
                BBody::Tree { tree, matrix } => tree.nearest(matrix, ray),
            };
            if let Some(d) = hit
                && best.is_none_or(|(_, b)| d < b)
            {
                best = Some((o.name, d));
            }
        }
        best.map(|(name, _)| name)
    }

    /// The grid's side for this scene: as fine as the budget allows.
    fn side(&self) -> usize {
        let rays = BUDGET / self.triangles().max(1);
        ((rays as f64).sqrt() as usize).clamp(4, MAX_SIDE)
    }

    /// Of `candidates` (projected bounds overlapping `boxed`), the ones a
    /// ray through the box sees first, in their order.
    pub fn visible(&self, view: &BPickView, boxed: BBox2, candidates: Vec<String>) -> Vec<String> {
        let (x0, x1) = (boxed.a.0.min(boxed.b.0), boxed.a.0.max(boxed.b.0));
        let (y0, y1) = (boxed.a.1.min(boxed.b.1), boxed.a.1.max(boxed.b.1));
        let n = self.side();
        let mut seen: HashSet<&str> = HashSet::new();
        for i in 0..n {
            for j in 0..n {
                let su = x0 + (x1 - x0) * (i as f32 + 0.5) / n as f32;
                let sv = y0 + (y1 - y0) * (j as f32 + 0.5) / n as f32;
                seen.extend(self.nearest(view.ray(su, sv)));
            }
        }
        for o in self.objects.iter().filter(|o| candidates.iter().any(|c| c == o.name)) {
            let centre = (o.lo + o.hi) * 0.5;
            if let Some((su, sv)) = view.project(centre)
                && (x0..=x1).contains(&su)
                && (y0..=y1).contains(&sv)
            {
                seen.extend(self.nearest(view.ray(su, sv)));
            }
        }
        candidates.into_iter().filter(|c| seen.contains(c.as_str())).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::scene::boxsel::boxed_names;
    use crate::v2::scene::camera::BViewSetup;
    use crate::v2::scene::BShape;

    fn cube(name: &str, x: f32, y: f32) -> BObject3D {
        BObject3D::Primitive { name: name.into(), shape: BShape::Cube, x: x * MESH_SCALE, y: y * MESH_SCALE }
    }

    #[test]
    fn a_box_takes_what_it_sees_not_what_is_hidden_behind() {
        // Yaw 0 looks along +Y: Front hides Back; Beside stands clear.
        let scene = [cube("Back", 0.0, 3.0), cube("Front", 0.0, -3.0), cube("Beside", 6.0, 0.0)];
        let view = BPickView { yaw: 0.0, pitch: 0.0, pan: (0.0, 0.0), zoom: 0.5, setup: BViewSetup::default(), aspect: 1.0 };
        let all = BBox2 { a: (-0.5, -0.5), b: (0.5, 0.5) };
        let overlapped = boxed_names(&scene, &view, all, |_| true);
        assert_eq!(overlapped, ["Back", "Front", "Beside"], "every projected box overlaps");
        let seen = BPickScene::new(&scene, |_| true).visible(&view, all, overlapped.clone());
        assert_eq!(seen, ["Front", "Beside"], "Back is behind Front");
        // Front's arrow toggle set: it is out of the pass, and Back shows through.
        let through = BPickScene::new(&scene, |n| n != "Front").visible(&view, all, overlapped.into_iter().filter(|n| n != "Front").collect());
        assert_eq!(through, ["Back", "Beside"]);
        // Wireframe is X-ray in a new file: there the box keeps everything
        // it overlaps; Solid is not until Alt Z.
        use crate::v2::scene::{BShading, BXray};
        assert!(BXray::FACTORY.active(BShading::Wireframe) && !BXray::FACTORY.active(BShading::Solid));
    }
}
