//! Picking: the object under a point of the 3D viewport.
//!
//! Blender ref: `view3d_select.cc` (`ed_object_select_pick` →
//! `mixed_bones_object_selectbuffer`), which draws an ID pass and reads the
//! pixel. Here the pixel becomes a ray — the same ray the tracer's kernels
//! build for it (`solid.wgsl`: screen → view → `pivot + back·dist`, so a
//! click lands on what the pixel shows) — and the ray is cast against every
//! visible object's triangles on the CPU, nearest hit wins. A box cull per
//! object keeps a big scene cheap enough for a click.

use crate::v2::scene::{BObject3D, MESH_SCALE};
use crate::v2::scene::camera::{BViewSetup, SCREEN};
use crate::phiphx::spatial::SpatialObject3D;
pub mod extra;
pub mod occlude;
pub mod tree;
mod triangle;
pub use triangle::triangle_t;
use triangle::hits_box;

use glam::{Mat4, Vec3};

/// The view a pick is made in: what the tracer's camera is built from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPickView {
    pub yaw: f32,
    pub pitch: f32,
    pub pan: (f32, f32),
    pub zoom: f32,
    pub setup: BViewSetup,
    /// Canvas width / height, as the tracer is given it.
    pub aspect: f32,
}

/// A ray in world units (Blender's, Z up): origin and unit direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPickRay {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl BPickView {
    /// The view the viewport under the pointer shows (`scene::view_slot`),
    /// on the canvas as last painted — for a reader with no canvas in hand.
    pub fn now(s: &crate::v2::state::BState) -> Self {
        use crate::v2::scene::{camera, view_pan, view_pitch, view_yaw, view_zoom};
        Self { yaw: view_yaw(s), pitch: view_pitch(s), pan: view_pan(s), zoom: view_zoom(s), setup: camera::setup_from_state(s), aspect: camera::last_aspect() }
    }

    /// The view's axes in world units: right, up, and back — from the view
    /// toward the eye.
    pub fn axes(&self) -> (Vec3, Vec3, Vec3) {
        let (sy, cy) = self.yaw.to_radians().sin_cos();
        let (sp, cp) = self.pitch.to_radians().sin_cos();
        (Vec3::new(cy, -sy, 0.0), Vec3::new(sp * sy, sp * cy, cp), Vec3::new(-sy * cp, -cy * cp, sp))
    }

    /// The ray through the canvas point `(su, sv)` — each in `-0.5 ..= 0.5`
    /// across the canvas, `sv` down — exactly as `solid.wgsl` builds it.
    pub fn ray(&self, su: f32, sv: f32) -> BPickRay {
        let (right, up, back) = self.axes();
        let scale = SCREEN * self.zoom;
        let vx = (su - self.pan.0) / scale;
        let vy = -(sv / self.aspect - self.pan.1) / scale;
        let ms = MESH_SCALE;
        let pivot = Vec3::new(self.setup.pivot.0 / ms, self.setup.pivot.2 / ms, self.setup.pivot.1 / ms);
        if self.setup.perspective {
            let dist = 1.0 / (scale * self.setup.fov);
            BPickRay { origin: pivot + back * (dist / ms), dir: (right * vx + up * vy - back * dist).normalize() }
        } else {
            BPickRay { origin: pivot + (right * vx + up * vy + back * 1.0e4) / ms, dir: -back }
        }
    }

    /// The point `(x, y)` pixels into a canvas `w × h` as canvas coordinates.
    pub fn canvas_point(x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
        (x / w.max(1.0) - 0.5, y / h.max(1.0) - 0.5)
    }

    /// Where `object`'s origin lands on a `w × h` canvas, in pixels — what
    /// circle and lasso select test (`ED_view3d_project_base`).
    pub fn origin_on_canvas(&self, object: &BObject3D, w: f32, h: f32) -> Option<(f32, f32)> {
        let origin = match object {
            BObject3D::Primitive { x, y, .. } => Vec3::new(x / MESH_SCALE, y / MESH_SCALE, 0.0),
            BObject3D::Loaded { matrix_world, .. } => Mat4::from_cols_array_2d(matrix_world).w_axis.truncate(),
        };
        self.project(origin).map(|(su, sv)| ((su + 0.5) * w, (sv + 0.5) * h))
    }

    /// Where a world point lands on the canvas, `(su, sv)` as [`Self::ray`]
    /// takes them — the ray's inverse. `None` behind the eye.
    pub fn project(&self, p: Vec3) -> Option<(f32, f32)> {
        let (right, up, back) = self.axes();
        let scale = SCREEN * self.zoom;
        let ms = MESH_SCALE;
        let pivot = Vec3::new(self.setup.pivot.0 / ms, self.setup.pivot.2 / ms, self.setup.pivot.1 / ms);
        let (vx, vy) = if self.setup.perspective {
            let dist = 1.0 / (scale * self.setup.fov);
            let d = p - (pivot + back * (dist / ms));
            let db = d.dot(back);
            if db >= 0.0 {
                return None;
            }
            (-dist * d.dot(right) / db, -dist * d.dot(up) / db)
        } else {
            let d = p - pivot;
            (d.dot(right) * ms, d.dot(up) * ms)
        };
        Some((vx * scale + self.pan.0, self.aspect * (self.pan.1 - vy * scale)))
    }
}

/// A hit: the object's name and how far along the ray it was.
#[derive(Clone, Debug, PartialEq)]
pub struct BPick {
    pub name: String,
    pub distance: f32,
}

/// The nearest object `ray` meets in `scene`, if any.
pub fn pick(scene: &[BObject3D], ray: BPickRay) -> Option<BPick> {
    pick_where(scene, ray, |_| true)
}

/// The nearest object along `ray` among those `allowed` — an object whose
/// arrow toggle is set (`OB_HIDE_SELECT`) lets the ray through to what is
/// behind it, as Blender's `BASE_SELECTABLE` test does.
pub fn pick_where(scene: &[BObject3D], ray: BPickRay, allowed: impl Fn(&str) -> bool) -> Option<BPick> {
    let mut best: Option<BPick> = None;
    for object in scene {
        if !allowed(object.name()) {
            continue;
        }
        let Some(distance) = nearest(object, ray) else { continue };
        if best.as_ref().is_none_or(|b| distance < b.distance) {
            best = Some(BPick { name: object.name().to_string(), distance });
        }
    }
    best
}

/// The nearest triangle of `object` along `ray`.
fn nearest(object: &BObject3D, ray: BPickRay) -> Option<f32> {
    match object {
        BObject3D::Primitive { shape, x, y, .. } => {
            let (v, idx, _) = SpatialObject3D::buffers(shape.mesh_type());
            let (ox, oy) = (x / MESH_SCALE, y / MESH_SCALE);
            let world: Vec<Vec3> = v.iter().map(|v| Vec3::new(ox + v.0, oy + v.2, v.1)).collect();
            nearest_in(&world, &idx, ray)
        }
        // A large mesh through its tree, in its own space (`tree.rs`); a
        // small one walked whole in the world.
        BObject3D::Loaded { mesh, matrix_world, .. } if mesh.indices.len() / 3 >= tree::WHOLE => tree::BPickTree::of(mesh).nearest(matrix_world, ray),
        BObject3D::Loaded { mesh, matrix_world, .. } => {
            let m = Mat4::from_cols_array_2d(matrix_world);
            let world: Vec<Vec3> = mesh.vertices.iter().map(|v| m.transform_point3(Vec3::from(*v))).collect();
            nearest_in(&world, &mesh.indices, ray)
        }
    }
}

fn nearest_in(world: &[Vec3], indices: &[u32], ray: BPickRay) -> Option<f32> {
    if world.is_empty() || !hits_box(world, ray) {
        return None;
    }
    nearest_triangle(world, indices, ray)
}

/// The nearest of `indices`' triangles along `ray`, the box already passed.
pub(super) fn nearest_triangle(world: &[Vec3], indices: &[u32], ray: BPickRay) -> Option<f32> {
    let mut best = f32::INFINITY;
    for t in indices.chunks_exact(3) {
        let [a, b, c] = [t[0] as usize, t[1] as usize, t[2] as usize];
        if a >= world.len() || b >= world.len() || c >= world.len() {
            continue;
        }
        if let Some(d) = triangle_t(world[a], world[b], world[c], ray) {
            best = best.min(d);
        }
    }
    best.is_finite().then_some(best)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn looking_down_x() -> BPickView {
        // Blender's Right view: yaw 90 looks along −X … the exact angles do
        // not matter here, only that the ray and the projection agree.
        BPickView { yaw: 90.0, pitch: 0.0, pan: (0.0, 0.0), zoom: 1.0, setup: BViewSetup::default(), aspect: 16.0 / 9.0 }
    }

    #[test]
    fn the_centre_pixels_ray_runs_from_the_eye_through_the_pivot() {
        let view = looking_down_x();
        let ray = view.ray(0.0, 0.0);
        // Orthographic: the direction is straight along −back, the origin far behind the pivot.
        let (sy, cy) = view.yaw.to_radians().sin_cos();
        let back = Vec3::new(-sy, -cy, 0.0);
        assert!((ray.dir + back).length() < 1e-5);
        assert!(ray.origin.dot(back) > 1000.0, "origin far back along the view");
        let mut persp = view;
        persp.setup.perspective = true;
        let ray = persp.ray(0.0, 0.0);
        assert!((ray.dir + back).length() < 1e-5, "the centre ray points at the pivot in perspective too");
    }

    #[test]
    fn projecting_a_point_gives_the_pixel_whose_ray_passes_through_it() {
        for perspective in [false, true] {
            let mut view = BPickView { yaw: 33.0, pitch: 20.0, pan: (0.1, -0.05), zoom: 1.3, setup: BViewSetup::default(), aspect: 1.5 };
            view.setup.perspective = perspective;
            for p in [Vec3::new(0.3, -0.2, 0.4), Vec3::new(-1.0, 0.5, 0.1)] {
                let (su, sv) = view.project(p).expect("in front of the eye");
                let ray = view.ray(su, sv);
                let along = (p - ray.origin).dot(ray.dir);
                let off = (ray.origin + ray.dir * along - p).length();
                // The orthographic ray starts 1e4 units back, which costs f32 a few thousandths.
                assert!(off < 0.02, "perspective={perspective}: the ray through ({su}, {sv}) misses {p} by {off}");
            }
        }
        let mut persp = looking_down_x();
        persp.setup.perspective = true;
        let eye = persp.ray(0.0, 0.0).origin;
        let behind = eye + (eye - Vec3::ZERO);
        assert_eq!(persp.project(behind), None, "behind the eye there is no pixel");
    }

    #[test]
    fn the_nearest_of_two_cubes_is_picked_and_empty_space_is_a_miss() {
        let near = BObject3D::Primitive { name: "Near".into(), shape: crate::v2::scene::BShape::Cube, x: 0.0, y: 0.0 };
        let far = BObject3D::Primitive { name: "Far".into(), shape: crate::v2::scene::BShape::Cube, x: 0.0, y: 3.0 * MESH_SCALE * 10.0 };
        let view = BPickView { yaw: 0.0, pitch: 0.0, pan: (0.0, 0.0), zoom: 1.0, setup: BViewSetup::default(), aspect: 1.0 };
        let ray = view.ray(0.0, 0.0);
        let picked = pick(&[far.clone(), near.clone()], ray).expect("a cube under the centre");
        assert_eq!(picked.name, "Near");
        assert_eq!(pick(&[near], view.ray(0.49, 0.49)).map(|p| p.name), None, "the corner of the canvas shows nothing");
    }
}
