// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/bvh/util.h`: `intersection_ray_valid`
//! (lines 13-27), and that a ray does not hit what it starts from (lines
//! 240-253). The rest of that file, offsetting a ray and sorting hits,
//! comes with shading and shadows.

use crate::kernel::{Ray, RaySelfPrimitive};
use crate::util::vector::dot;

/// `intersection_ray_valid`: whether there is a ray to walk the tree
/// with. A start or a direction that is not a number would enter every
/// box; an empty ray stands for a trace that was not wanted.
pub fn intersection_ray_valid(ray: &Ray) -> bool {
    ray.p[0].is_finite() && ray.d[0].is_finite() && dot(ray.d, ray.d) != 0.0 && ray.tmin < f32::MAX
}

/// `intersection_skip_self`: whether this primitive is the one the ray
/// starts from.
pub fn intersection_skip_self(own: &RaySelfPrimitive, object: i32, prim: i32) -> bool {
    own.prim == prim && own.object == object
}

/// `intersection_skip_self_shadow`: that, or the light the ray goes to.
pub fn intersection_skip_self_shadow(own: &RaySelfPrimitive, object: i32, prim: i32) -> bool {
    intersection_skip_self(own, object, prim) || (own.light_prim == prim && own.light_object == object)
}
