// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/util/math_intersect.h`
//! (`ray_triangle_intersect`, lines 116-222): where a ray meets a triangle.
//! The spheres, disks and quads of that file come with the lights.

use super::vector::{add, cross, dot, sub};

/// `ray_triangle_intersect`: the ray from `ray_p` along `ray_d` against
/// the triangle `a`, `b`, `c`, between `tmin` and `tmax` along the ray.
/// The answer is `u`, `v` and `t`: the weights of `b` and `c` at the hit
/// (the rest is `a`'s) and how far along the ray it is.
///
/// As in Cycles this is the Plücker test Embree uses: the ray is tested
/// against the three edges, so two triangles that share an edge leave no
/// gap between them.
pub fn ray_triangle_intersect(ray_p: [f32; 3], ray_d: [f32; 3], tmin: f32, tmax: f32, a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Option<[f32; 3]> {
    // The corners from the ray's start, and the edges between them.
    let (v0, v1, v2) = (sub(a, ray_p), sub(b, ray_p), sub(c, ray_p));
    let (e0, e1, e2) = (sub(v2, v0), sub(v0, v1), sub(v1, v2));
    // Which side of each edge the ray passes.
    let u = dot(cross(e0, add(v2, v0)), ray_d);
    let v = dot(cross(e1, add(v0, v1)), ray_d);
    let w = dot(cross(e2, add(v1, v2)), ray_d);
    let uvw = u + v + w;
    let eps = f32::EPSILON * uvw.abs();
    let (least, most) = (u.min(v.min(w)), u.max(v.max(w)));
    if !(least >= -eps || most <= eps) {
        return None;
    }
    // The plane of the triangle, and how far along the ray it is.
    let normal = cross(e1, e0);
    let normal = add(normal, normal);
    let den = dot(normal, ray_d);
    if den == 0.0 {
        return None;
    }
    let t = dot(v0, normal) / den;
    if !(t >= tmin && t <= tmax) {
        return None;
    }
    // Cycles takes the reciprocal with the processor's own instruction and
    // a step of refinement; this is the division it stands for.
    let inverse = match uvw.abs() < 1e-18 {
        true => 0.0,
        false => 1.0 / uvw,
    };
    Some([(u * inverse).min(1.0), (v * inverse).min(1.0), t])
}
