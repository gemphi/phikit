// SPDX-FileCopyrightText: 2014-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/geom/triangle_intersect.h`
//! (`triangle_intersect`, lines 24-62). The local intersection, for
//! subsurface scattering, and the refinement of a hit come later.
use super::triangle_vertices;
use crate::kernel::{Intersection, KernelGlobal};
use crate::util::ray_triangle_intersect;
/// `triangle_intersect`: the ray from `p` along `dir` against triangle
/// `prim` of `object`, which is entry `prim_addr` of the tree's
/// primitives. A hit between `tmin` and `tmax` that a ray of this
/// `visibility` may see is written into `isect`.
#[allow(clippy::too_many_arguments, reason = "the arguments Cycles gives it")]
pub fn triangle_intersect(kg: &KernelGlobal, isect: &mut Intersection, p: [f32; 3], dir: [f32; 3], tmin: f32, tmax: f32, visibility: u32, object: i32, prim: i32, prim_addr: i32) -> bool {
    let [a, b, c] = triangle_vertices(kg, object, prim);
    let Some(hit) = ray_triangle_intersect(p, dir, tmin, tmax, a, b, c) else {
        return false;
    };
    // The flag is tested after the hit: most triangles a ray may not see
    // were left out with their node.
    if kg.scene.prim_visibility[prim_addr as usize] & visibility == 0 {
        return false;
    }
    *isect = Intersection { t: hit.t, u: hit.u, v: hit.v, prim, object, kind: kg.bvh.primitive_triangle as i32 };
    true
}
