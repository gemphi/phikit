//! The ray against the geometry: a box cull per object, then Möller–Trumbore
//! per triangle — the same test `bvh.wgsl::tri_t` makes on the GPU.

use super::BPickRay;
use glam::Vec3;

/// Does the ray pass through the vertices' bounding box?
pub(super) fn hits_box(world: &[Vec3], ray: BPickRay) -> bool {
    let (lo, hi) = bounds(world);
    hits_bounds(lo, hi, ray)
}

/// The vertices' bounding box, `(lo, hi)`.
pub(super) fn bounds(world: &[Vec3]) -> (Vec3, Vec3) {
    world.iter().fold((Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)), |(lo, hi), v| (lo.min(*v), hi.max(*v)))
}

/// Does the ray pass through the box `lo..hi` in front of its origin?
pub(super) fn hits_bounds(lo: Vec3, hi: Vec3, ray: BPickRay) -> bool {
    let inv = ray.dir.map(|d| if d.abs() < 1e-12 { 1.0 / 1e-12 } else { 1.0 / d });
    let t0 = (lo - ray.origin) * inv;
    let t1 = (hi - ray.origin) * inv;
    let near = t0.min(t1).max_element();
    let far = t0.max(t1).min_element();
    far >= near.max(0.0)
}

/// Möller–Trumbore, two-sided, as `bvh.wgsl::tri_t`: the distance along the
/// ray to the triangle, when it is in front of the origin.
pub fn triangle_t(a: Vec3, b: Vec3, c: Vec3, ray: BPickRay) -> Option<f32> {
    let e1 = b - a;
    let e2 = c - a;
    let p = ray.dir.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-9 {
        return None;
    }
    let inv = 1.0 / det;
    let s = ray.origin - a;
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = ray.dir.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (t > 1e-6).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_triangle_in_front_is_hit_and_one_behind_is_not() {
        let ray = BPickRay { origin: Vec3::ZERO, dir: Vec3::X };
        let ahead = triangle_t(Vec3::new(2.0, -1.0, -1.0), Vec3::new(2.0, 1.0, -1.0), Vec3::new(2.0, 0.0, 1.0), ray);
        assert!((ahead.unwrap() - 2.0).abs() < 1e-6);
        let behind = triangle_t(Vec3::new(-2.0, -1.0, -1.0), Vec3::new(-2.0, 1.0, -1.0), Vec3::new(-2.0, 0.0, 1.0), ray);
        assert_eq!(behind, None);
        let beside = triangle_t(Vec3::new(2.0, 3.0, -1.0), Vec3::new(2.0, 5.0, -1.0), Vec3::new(2.0, 4.0, 1.0), ray);
        assert_eq!(beside, None);
    }
}
