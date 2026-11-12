// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/util/math_intersect.h`
//! (`ray_triangle_intersect`, line 163; `ray_triangle_intersect_self`,
//! line 252): the Plücker-coordinate triangle test of Embree, which Cycles'
//! own BVH uses. The scalar forms of `ray_triangle_dot`, `_cross` and
//! `_reciprocal` are used, those of the build without SSE4.2.
use super::vector::{add, cross, dot, sub};

/// A hit: the barycentric coordinates and the distance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriangleHit {
    pub u: f32,
    pub v: f32,
    pub t: f32,
}

/// `ray_triangle_intersect`: does the ray from `p` along `d` cross the
/// triangle `a`, `b`, `c` between `tmin` and `tmax`?
pub fn ray_triangle_intersect(p: [f32; 3], d: [f32; 3], tmin: f32, tmax: f32, a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Option<TriangleHit> {
    // The vertices relative to the ray origin, and the triangle's edges.
    let v0 = sub(a, p);
    let v1 = sub(b, p);
    let v2 = sub(c, p);
    let e0 = sub(v2, v0);
    let e1 = sub(v0, v1);
    let e2 = sub(v1, v2);
    // The edge tests.
    let u = dot(cross(e0, add(v2, v0)), d);
    let v = dot(cross(e1, add(v0, v1)), d);
    let w = dot(cross(e2, add(v1, v2)), d);
    let uvw = u + v + w;
    let eps = f32::EPSILON * uvw.abs();
    let min_uvw = u.min(v.min(w));
    let max_uvw = u.max(v.max(w));
    if !(min_uvw >= -eps || max_uvw <= eps) {
        return None;
    }
    // The geometry normal and the denominator; not parallel.
    let ng1 = cross(e1, e0);
    let ng = add(ng1, ng1);
    let den = dot(ng, d);
    if den == 0.0 {
        return None;
    }
    // The depth test.
    let t = dot(v0, ng) / den;
    if !(t >= tmin && t <= tmax) {
        return None;
    }
    let rcp_uvw = if uvw.abs() < 1e-18 { 0.0 } else { 1.0 / uvw };
    Some(TriangleHit { u: (u * rcp_uvw).min(1.0), v: (v * rcp_uvw).min(1.0), t })
}

/// `ray_triangle_intersect_self`: whether a ray from a point on the
/// triangle is going to hit it again, or might hit a neighbour instead.
/// The test of `ray_triangle_intersect` with an epsilon wider for
/// neighbours that have one.
pub fn ray_triangle_intersect_self(p: [f32; 3], d: [f32; 3], verts: [[f32; 3]; 3]) -> bool {
    let v0 = sub(verts[0], p);
    let v1 = sub(verts[1], p);
    let v2 = sub(verts[2], p);
    let e0 = sub(v2, v0);
    let e1 = sub(v0, v1);
    let e2 = sub(v1, v2);
    let u = dot(cross(add(v2, v0), e0), d);
    let v = dot(cross(add(v0, v1), e1), d);
    let w = dot(cross(add(v1, v2), e2), d);
    let eps = f32::EPSILON * (u + v + w).abs();
    let min_uvw = u.min(v.min(w));
    let max_uvw = u.max(v.max(w));
    min_uvw >= eps || max_uvw <= -eps
}

#[cfg(test)]
mod test;
