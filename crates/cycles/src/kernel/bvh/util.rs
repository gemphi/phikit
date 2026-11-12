// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/bvh/util.h`: `intersection_ray_valid`
//! (lines 13-27), `intersection_t_offset` (35), `ray_offset` (66), and that
//! a ray does not hit what it starts from (lines 240-253). Sorting hits
//! comes with the shadow walk that needs it.
use crate::kernel::{Ray, RaySelfPrimitive};
use crate::util::vector::dot;
/// `intersection_ray_valid`: whether there is a ray to walk the tree
/// with. A start or a direction that is not a number would enter every
/// box; an empty ray stands for a trace that was not wanted.
pub fn intersection_ray_valid(ray: &Ray) -> bool {
    ray.p[0].is_finite() && ray.d[0].is_finite() && dot(ray.d, ray.d) != 0.0 && ray.tmin < f32::MAX
}
/// `intersection_t_offset`: the next float above `t`, so that a hit at `t`
/// is skipped. Zero gives the smallest normal float.
pub fn intersection_t_offset(t: f32) -> f32 {
    match t == 0.0 {
        true => f32::MIN_POSITIVE,
        false => f32::from_bits(t.to_bits() + 1),
    }
}
/// `ray_offset` (Ray Tracing Gems, chapter 6): a start position for a ray
/// leaving the surface at `p` with geometry normal `ng`.
pub fn ray_offset(p: [f32; 3], ng: [f32; 3]) -> [f32; 3] {
    let int_scale = 256.0f32;
    let origin = 1.0f32 / 32.0;
    let float_scale = 1.0f32 / 65536.0;
    std::array::from_fn(|axis| {
        let of_i = (int_scale * ng[axis]) as i32;
        let bits = p[axis].to_bits() as i32;
        let p_i = f32::from_bits(bits.wrapping_add(if p[axis] < 0.0 { -of_i } else { of_i }) as u32);
        if p[axis].abs() < origin { p[axis] + float_scale * ng[axis] } else { p_i }
    })
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

#[cfg(test)]
mod test;
