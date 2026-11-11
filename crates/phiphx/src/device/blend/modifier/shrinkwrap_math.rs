//! The `BLI_math_geom` primitives the Shrinkwrap uses, translated one for one
//! so its nearest points, hits and normals round the way Blender's do.

use glam::Vec3;

/// `normalize_v3`: a zero vector stays zero.
pub(super) fn normalize(v: Vec3) -> Vec3 {
    v.normalize_or_zero()
}

/// `interp_v3_v3v3`: `(1 - t)·a + t·b`.
pub(super) fn interp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    let s = 1.0 - t;
    a * s + b * t
}

/// `normal_tri_v3`: `(v1 − v2) × (v2 − v3)`, normalized.
pub(super) fn normal_tri(v1: Vec3, v2: Vec3, v3: Vec3) -> Vec3 {
    normalize((v1 - v2).cross(v2 - v3))
}

/// `closest_on_tri_to_point_v3` (Ericson, *Real-Time Collision Detection*).
pub(super) fn closest_on_tri(p: Vec3, v1: Vec3, v2: Vec3, v3: Vec3) -> Vec3 {
    let (ab, ac, ap) = (v2 - v1, v3 - v1, p - v1);
    let (d1, d2) = (ab.dot(ap), ac.dot(ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return v1;
    }
    let bp = p - v2;
    let (d3, d4) = (ab.dot(bp), ac.dot(bp));
    if d3 >= 0.0 && d4 <= d3 {
        return v2;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let ab_squared = d1 - d3;
        return match ab_squared == 0.0 {
            true => v1,
            false => v1 + ab * (d1 / ab_squared),
        };
    }
    let cp = p - v3;
    let (d5, d6) = (ab.dot(cp), ac.dot(cp));
    if d6 >= 0.0 && d5 <= d6 {
        return v3;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let ac_squared = d2 - d6;
        return match ac_squared == 0.0 {
            true => v1,
            false => v1 + ac * (d2 / ac_squared),
        };
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let bc_squared = (d4 - d3) + (d5 - d6);
        return match bc_squared == 0.0 {
            true => v2,
            false => (v3 - v2) * ((d4 - d3) / bc_squared) + v2,
        };
    }
    let denom = 1.0 / (va + vb + vc);
    let (v, w) = (vb * denom, vc * denom);
    let acw = ac * w;
    (v1 + ab * v) + acw
}

/// `isect_ray_tri_watertight_v3_precalc`: the shear of one ray.
#[derive(Clone, Copy)]
pub(super) struct RayPrecalc {
    k: [usize; 3],
    s: [f32; 3],
}

impl RayPrecalc {
    pub(super) fn new(dir: Vec3) -> Self {
        // `axis_dominant_v3_single`.
        let (x, y, z) = (dir.x.abs(), dir.y.abs(), dir.z.abs());
        let kz = match x > y {
            true => match x > z { true => 0, false => 2 },
            false => match y > z { true => 1, false => 2 },
        };
        let kx = match kz != 2 { true => kz + 1, false => 0 };
        let ky = match kx != 2 { true => kx + 1, false => 0 };
        // Swap to preserve the winding when the ray points down its axis.
        let (kx, ky) = match dir[kz] < 0.0 { true => (ky, kx), false => (kx, ky) };
        let inv_dir_z = 1.0 / dir[kz];
        Self { k: [kx, ky, kz], s: [dir[kx] * inv_dir_z, dir[ky] * inv_dir_z, inv_dir_z] }
    }
}

/// `isect_ray_tri_watertight_v3`: the hit distance along the ray, if any.
pub(super) fn ray_tri_watertight(origin: Vec3, pre: &RayPrecalc, v0: Vec3, v1: Vec3, v2: Vec3) -> Option<f32> {
    let [kx, ky, kz] = pre.k;
    let [sx, sy, sz] = pre.s;
    let (a, b, c) = (v0 - origin, v1 - origin, v2 - origin);
    let (ax, ay) = (a[kx] - sx * a[kz], a[ky] - sy * a[kz]);
    let (bx, by) = (b[kx] - sx * b[kz], b[ky] - sy * b[kz]);
    let (cx, cy) = (c[kx] - sx * c[kz], c[ky] - sy * c[kz]);
    let u = cx * by - cy * bx;
    let v = ax * cy - ay * cx;
    let w = bx * ay - by * ax;
    if (u < 0.0 || v < 0.0 || w < 0.0) && (u > 0.0 || v > 0.0 || w > 0.0) {
        return None;
    }
    let det = u + v + w;
    if det == 0.0 || !det.is_finite() {
        return None;
    }
    let t = (u * a[kz] + v * b[kz] + w * c[kz]) * sz;
    // `xor_fl(t, sign_det)`: t with det's sign bit folded in.
    let sign_t = match det.is_sign_negative() { true => -t, false => t };
    if sign_t < 0.0 {
        return None;
    }
    Some(t * (1.0 / det))
}

/// `interp_weights_tri_v3` (`barycentric_weights` over the dominant plane).
pub(super) fn interp_weights_tri(v1: Vec3, v2: Vec3, v3: Vec3, co: Vec3) -> [f32; 3] {
    let n = normal_tri(v1, v2, v3);
    // `axis_dominant_v3`.
    let (xn, yn, zn) = (n.x.abs(), n.y.abs(), n.z.abs());
    let (i, j) = match (zn >= xn && zn >= yn, yn >= xn && yn >= zn) {
        (true, _) => (0, 1),
        (false, true) => (0, 2),
        (false, false) => (1, 2),
    };
    // `tri_signed_area`.
    let area = |a: Vec3, b: Vec3, c: Vec3| 0.5 * ((a[i] - b[i]) * (b[j] - c[j]) + (a[j] - b[j]) * (c[i] - b[i]));
    let w = [area(v2, v3, co), area(v3, v1, co), area(v1, v2, co)];
    let inv = 1.0 / (w[0] + w[1] + w[2]);
    let w = [w[0] * inv, w[1] * inv, w[2] * inv];
    match w.iter().all(|x| x.is_finite()) {
        true => w,
        false => [1.0 / 3.0; 3],
    }
}

/// `math::safe_acos_approx`, the corner angle `normals_calc_verts` weights by.
fn safe_acos_approx(x: f32) -> f32 {
    let f = x.abs();
    let m = match f < 1.0 { true => 1.0 - (1.0 - f), false => 1.0 };
    let a = (1.0 - m).sqrt() * (1.570_796_3 + m * (-0.213_300_99 + m * (0.077_980_48 + m * -0.021_640_95)));
    match x < 0.0 { true => std::f32::consts::PI - a, false => a }
}

/// `face_normal_calc`: the triangle's own normal, Newell's for the rest
/// (`normal_quad_v3`'s diagonal cross is the same direction for a quad).
pub(super) fn face_normal(positions: &[Vec3], face: &[u32]) -> Vec3 {
    let at = |k: usize| positions[face[k] as usize];
    match face.len() {
        0..=2 => Vec3::ZERO,
        3 => normal_tri(at(0), at(1), at(2)),
        n => {
            let mut sum = Vec3::ZERO;
            for k in 0..n {
                let (a, b) = (at(k), at((k + 1) % n));
                sum += Vec3::new((a.y - b.y) * (a.z + b.z), (a.z - b.z) * (a.x + b.x), (a.x - b.x) * (a.y + b.y));
            }
            normalize(sum)
        }
    }
}

/// `mesh::normals_calc_verts`: face normals weighted by each corner's angle;
/// a vertex no face uses takes its own position, normalized.
pub(super) fn vertex_normals<'a>(positions: &[Vec3], faces: impl Iterator<Item = &'a [u32]>, face_normals: &[Vec3]) -> Vec<Vec3> {
    let mut out = vec![Vec3::ZERO; positions.len()];
    let mut used = vec![false; positions.len()];
    for (f, face) in faces.enumerate() {
        let n = face.len();
        for k in 0..n {
            let v = face[k] as usize;
            let prev = positions[face[(k + n - 1) % n] as usize];
            let next = positions[face[(k + 1) % n] as usize];
            let dir_prev = normalize(prev - positions[v]);
            let dir_next = normalize(next - positions[v]);
            out[v] += face_normals[f] * safe_acos_approx(dir_prev.dot(dir_next));
            used[v] = true;
        }
    }
    for (v, n) in out.iter_mut().enumerate() {
        *n = match used[v] {
            true => normalize(*n),
            false => normalize(positions[v]),
        };
    }
    out
}

/// `mesh_calc_tessellation_for_face_impl`'s quad case: (0,1,2)+(0,2,3),
/// or (0,1,3)+(1,2,3) when `is_quad_flip_v3_first_third_fast` says the
/// 0–2 diagonal is degenerate.
pub(super) fn quad_tris(v: [Vec3; 4]) -> [[usize; 3]; 2] {
    let (d12, d13, d14) = (v[1] - v[0], v[2] - v[0], v[3] - v[0]);
    match d12.cross(d13).dot(d14.cross(d13)) > 0.0 {
        true => [[0, 1, 3], [1, 2, 3]],
        false => [[0, 1, 2], [0, 2, 3]],
    }
}
