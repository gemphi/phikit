// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::kernel::RaySelfPrimitive;

#[test]
fn a_ray_is_valid_when_it_starts_and_goes_somewhere() {
    let ray = along_z([0.0; 3], 1.0);
    assert!(intersection_ray_valid(&ray));
    assert!(!intersection_ray_valid(&Ray::default()), "a ray with no direction");
    assert!(!intersection_ray_valid(&Ray { p: [f32::INFINITY, 0.0, 0.0], ..ray }));
    assert!(!intersection_ray_valid(&Ray { tmin: f32::MAX, ..ray }));
}

#[test]
fn what_a_ray_starts_from_is_a_primitive_of_an_object() {
    let own = RaySelfPrimitive { prim: 3, object: 1, light_prim: 8, light_object: 2 };
    assert!(intersection_skip_self(&own, 1, 3));
    assert!(!intersection_skip_self(&own, 1, 4) && !intersection_skip_self(&own, 0, 3));
    // A shadow ray skips the light it goes to as well.
    assert!(intersection_skip_self_shadow(&own, 1, 3) && intersection_skip_self_shadow(&own, 2, 8));
    assert!(!intersection_skip_self(&own, 2, 8) && !intersection_skip_self_shadow(&own, 1, 8));
    // A camera's ray starts from nothing, and skips nothing.
    assert!(!intersection_skip_self_shadow(&RaySelfPrimitive::default(), 0, 0));
}
#[test]
fn the_offset_distance_is_the_next_float() {
    assert_eq!(intersection_t_offset(0.0), f32::MIN_POSITIVE);
    for t in [1.0f32, 3.5, 1e-3, 123_456.0] {
        let next = intersection_t_offset(t);
        assert!(next > t && next.to_bits() == t.to_bits() + 1);
    }
}
#[test]
fn the_ray_offset_moves_a_point_along_the_normal_and_barely() {
    let moved = ray_offset([2.0, -3.0, 4.0], [0.0, 0.0, 1.0]);
    assert!(moved[2] > 4.0 && (moved[2] - 4.0) < 1e-3 && moved[0] == 2.0 && moved[1] == -3.0);
    // Near the origin it is a fixed small step.
    let near = ray_offset([0.0; 3], [0.0, 1.0, 0.0]);
    assert_eq!(near, [0.0, 1.0 / 65536.0, 0.0]);
    // A negative coordinate moves away from zero for a normal pointing the
    // same way as its sign.
    assert!(ray_offset([-2.0, 0.0, 0.0], [-1.0, 0.0, 0.0])[0] < -2.0);
}