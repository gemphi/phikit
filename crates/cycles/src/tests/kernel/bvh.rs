// SPDX-License-Identifier: Apache-2.0
//! Tests of `kernel/bvh/`: the closest hit through the scene's tree and an
//! instance's, and the helpers beside the walk.
use super::rig::{Rig, plane, translation};
use crate::kernel::bvh::*;
use crate::kernel::types::*;
use crate::scene::Surface;

fn ray(p: [f32; 3], d: [f32; 3]) -> Ray {
    Ray { p, d, tmin: 0.0, tmax: 1000.0, ..Ray::default() }
}

fn trace(kg: &crate::kernel::KernelGlobals, ray: &Ray) -> (bool, Intersection) {
    let mut isect = Intersection::default();
    let hit = scene_intersect(kg, ray, PATH_RAY_VISIBILITY_CAMERA, &mut isect);
    (hit, isect)
}

/// A plane at the origin and the same mesh placed twice more, each of a
/// side of 2: so the mesh is an instance.
fn rig() -> Rig {
    let mut rig = Rig::new(8, 8, [0.1; 3]);
    let mesh = rig.add(&plane(1.0, 0.0), Surface::Diffuse { color: [0.5; 3] }, translation(0.0, 0.0, 0.0));
    rig.instance(mesh, translation(5.0, 0.0, -2.0));
    rig.instance(mesh, translation(0.0, 0.0, 3.0));
    rig
}

#[test]
fn a_ray_hits_the_nearest_triangle_with_its_barycentric_coordinates() {
    let kg = rig().kernel();
    // The plane at z = 3 is the nearest from above; (0.5, -0.25) is in
    // the triangle (-1,-1) (1,-1) (1,1) at u = v = 0.375.
    let (hit, isect) = trace(&kg, &ray([0.5, -0.25, 8.0], [0.0, 0.0, -1.0]));
    assert!(hit);
    assert!((isect.t - 5.0).abs() < 1e-5 && (isect.u - 0.375).abs() < 1e-5 && (isect.v - 0.375).abs() < 1e-5, "{isect:?}");
    assert_eq!((isect.kind, isect.object), (PRIMITIVE_TRIANGLE as i32, 2));
}

#[test]
fn the_nearest_of_the_instances_wins_from_either_side() {
    let kg = rig().kernel();
    let (_, above) = trace(&kg, &ray([0.0, 0.5, 8.0], [0.0, 0.0, -1.0]));
    let (_, below) = trace(&kg, &ray([0.0, 0.5, -8.0], [0.0, 0.0, 1.0]));
    assert_eq!((above.object, below.object), (2, 0));
    assert!((above.t - 5.0).abs() < 1e-5 && (below.t - 8.0).abs() < 1e-5);
}

#[test]
fn a_ray_enters_an_instance_through_the_inverse_of_its_transform() {
    let kg = rig().kernel();
    // The instance at (5, 0, -2): the same mesh, in another place. A ray
    // over it hits it at the distance in the world.
    let (hit, isect) = trace(&kg, &ray([5.5, 0.25, 1.0], [0.0, 0.0, -1.0]));
    assert!(hit && isect.object == 1 && (isect.t - 3.0).abs() < 1e-5, "{isect:?}");
    // The point it hits in object space is the same as for the original:
    // (0.5, 0.25) in the plane, u = v = ... of the triangle it is in.
    let (_, home) = trace(&kg, &ray([0.5, 0.25, 1.0], [0.0, 0.0, -1.0]));
    assert!(((isect.u, isect.v) == (home.u, home.v)) && isect.prim == home.prim);
}

#[test]
fn a_ray_that_misses_or_runs_short_hits_nothing() {
    let kg = rig().kernel();
    let (hit, isect) = trace(&kg, &ray([9.0, 9.0, 8.0], [0.0, 0.0, -1.0]));
    assert!(!hit && isect.prim == NONE && isect.object == NONE);
    let short = Ray { tmax: 4.0, ..ray([0.0, 0.5, 8.0], [0.0, 0.0, -1.0]) };
    assert!(!trace(&kg, &short).0);
    // Beyond tmin.
    let past = Ray { tmin: 5.5, ..ray([0.0, 0.5, 8.0], [0.0, 0.0, -1.0]) };
    assert!(trace(&kg, &past).1.object == 0 && (trace(&kg, &past).1.t - 8.0).abs() < 1e-5);
}

#[test]
fn a_ray_does_not_hit_the_primitive_it_left() {
    let kg = rig().kernel();
    let (_, first) = trace(&kg, &ray([0.0, 0.5, 8.0], [0.0, 0.0, -1.0]));
    // From the hit point, along the same line: the same triangle is
    // skipped, and the next is the plane at the origin below it.
    let mut again = ray([0.0, 0.5, 3.0], [0.0, 0.0, -1.0]);
    again.kind = RaySelf { object: first.object, prim: first.prim, ..RaySelf::default() };
    let (hit, second) = trace(&kg, &again);
    assert!(hit && second.object == 0 && (second.t - 3.0).abs() < 1e-5);
}

#[test]
fn a_ray_with_no_direction_or_an_infinite_origin_is_not_traced() {
    assert!(intersection_ray_valid(&ray([0.0; 3], [0.0, 0.0, 1.0])));
    assert!(!intersection_ray_valid(&ray([0.0; 3], [0.0; 3])));
    assert!(!intersection_ray_valid(&ray([f32::INFINITY, 0.0, 0.0], [0.0, 0.0, 1.0])));
    assert!(!intersection_ray_valid(&Ray { tmin: f32::MAX, ..ray([0.0; 3], [0.0, 0.0, 1.0]) }));
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
    assert!(intersection_skip_self_shadow(&RaySelf { object: 1, prim: 2, ..RaySelf::default() }, 1, 2));
    assert!(!intersection_skip_self_shadow(&RaySelf { object: 1, prim: 2, ..RaySelf::default() }, 2, 2));
    assert!(intersection_skip_self_shadow(&RaySelf { light_object: 3, light_prim: 4, ..RaySelf::default() }, 3, 4));
}
