// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::tests::kernel::near;

#[test]
fn a_ray_hits_the_triangle_in_its_way_and_says_where() {
    let rig = rig_of(vec![square()]);
    // The square's first triangle is 0,0 1,0 1,1; the second 0,0 1,1 0,1.
    let first = hit(&rig, &along_z([0.75, 0.25, -2.0], 10.0), all()).expect("the square is in the way");
    let second = hit(&rig, &along_z([0.25, 0.75, -2.0], 10.0), all()).expect("the square is in the way");
    assert_eq!((first.prim, first.object, first.kind), (0, 0, flag("PRIMITIVE_TRIANGLE") as i32));
    assert_eq!((second.prim, second.object), (1, 0));
    // The weights of the second and the third corner: the hit is their
    // mix with the first, and it is two units along the ray.
    assert!(near(&[first.t, first.u, first.v], &[2.0, 0.5, 0.25]), "{first:?}");
    assert!(near(&[second.t, second.u, second.v], &[2.0, 0.25, 0.5]), "{second:?}");
}

#[test]
fn a_ray_past_the_mesh_or_too_short_hits_nothing() {
    let rig = rig_of(vec![square()]);
    assert!(hit(&rig, &along_z([1.5, 0.5, -2.0], 10.0), all()).is_none());
    assert!(hit(&rig, &along_z([0.5, 0.5, -2.0], 1.5), all()).is_none());
    // And one that starts beyond it, or looks the other way.
    assert!(hit(&rig, &Ray { tmin: 2.5, ..along_z([0.5, 0.5, -2.0], 10.0) }, all()).is_none());
    assert!(hit(&rig, &Ray { d: [0.0, 0.0, -1.0], ..along_z([0.5, 0.5, -2.0], 10.0) }, all()).is_none());
}

#[test]
fn of_several_things_in_the_way_the_nearest_is_hit() {
    // Three squares one behind the other, as three meshes.
    let lifted = |z: f32| {
        let mut mesh = square();
        mesh.geometry.position_mut().iter_mut().for_each(|point| point[2] = z);
        mesh
    };
    let rig = rig_of(vec![lifted(5.0), lifted(1.0), lifted(3.0)]);
    let nearest = hit(&rig, &along_z([0.25, 0.5, 0.0], 100.0), all()).expect("three squares are in the way");
    assert!(near(&[nearest.t], &[1.0]), "{nearest:?}");
    assert_eq!(nearest.object, 1);
    // From behind, the far one is the near one.
    let back = Ray { d: [0.0, 0.0, -1.0], ..along_z([0.25, 0.5, 9.0], 100.0) };
    let behind = hit(&rig, &back, all()).expect("three squares are in the way");
    assert!(behind.object == 0 && near(&[behind.t], &[4.0]), "{behind:?}");
    // Starting past the first two, only the last is left.
    let late = Ray { tmin: 3.5, ..along_z([0.25, 0.5, 0.0], 100.0) };
    assert_eq!(hit(&rig, &late, all()).map(|isect| isect.object), Some(0));
}

#[test]
fn every_square_of_a_long_strip_is_found_and_the_gaps_are_not() {
    let rig = rig_of(vec![strip(64)]);
    for at in 0..64 {
        let x = at as f32 * 2.0;
        let found = hit(&rig, &along_z([x + 0.75, 0.25, -1.0], 10.0), all());
        assert_eq!(found.map(|isect| isect.prim), Some(at * 2), "square {at}");
        assert!(hit(&rig, &along_z([x + 1.5, 0.25, -1.0], 10.0), all()).is_none(), "the gap after square {at}");
    }
    // A slanted ray too: from above the first square to the last.
    let slanted = Ray { p: [0.5, 0.75, -1.0], d: crate::util::vector::normalize([126.0, 0.0, 1.0]), tmax: 1000.0, ..Ray::default() };
    assert_eq!(hit(&rig, &slanted, all()).map(|isect| isect.prim), Some(63 * 2 + 1));
}

#[test]
fn a_ray_enters_an_instance_as_the_object_sees_it() {
    let far = Transform::translate(10.0, 0.0, 4.0);
    let big = Transform::translate(-20.0, 0.0, 0.0) * Transform::scale(4.0, 4.0, 1.0);
    let rig = instanced(&[Transform::IDENTITY, far, big]);
    assert!(!rig.mesh[0].geometry.transform_applied, "an instanced mesh keeps its own tree");
    let at = |x: f32, y: f32| hit(&rig, &along_z([x, y, -1.0], 100.0), all());
    let found = [at(0.75, 0.25), at(10.75, 0.25), at(-17.0, 3.5)].map(|isect| isect.expect("a square is in the way"));
    assert_eq!(found.map(|isect| (isect.object, isect.prim)), [(0, 0), (1, 0), (2, 1)]);
    // The distance is along the ray in the world, whatever the object's
    // size or place.
    assert!(near(&found.map(|isect| isect.t), &[1.0, 5.0, 1.0]), "{found:?}");
    assert!(at(5.0, 0.5).is_none() && at(-17.0, 4.5).is_none());
}

#[test]
fn a_ray_does_not_hit_what_it_starts_from() {
    let rig = rig_of(vec![square()]);
    let mut ray = along_z([0.75, 0.25, -2.0], 10.0);
    (ray.own.prim, ray.own.object) = (0, 0);
    assert!(hit(&rig, &ray, all()).is_none());
    // The same triangle of another object is another triangle.
    ray.own.object = 5;
    assert!(hit(&rig, &ray, all()).is_some());
}

#[test]
fn a_ray_sees_only_what_is_visible_to_its_kind() {
    let mut rig = Rig::new(vec![square()]);
    let camera = flag("PATH_RAY_VISIBILITY_CAMERA");
    rig.object[0].node.set("visibility", crate::graph::Value::Uint(all() & !camera));
    rig.update();
    let ray = along_z([0.75, 0.25, -2.0], 10.0);
    assert!(hit(&rig, &ray, camera).is_none());
    assert!(hit(&rig, &ray, flag("PATH_RAY_VISIBILITY_DIFFUSE")).is_some());
}
/// The joined scene of `tests/kernel/rig.rs`: a plane at the origin and
/// the same mesh placed twice more, each of a side of 2, so the mesh is an
/// instance.
fn placed() -> crate::tests::kernel::rig::Rig {
    use crate::tests::kernel::rig::{Rig, plane, translation};
    let mut rig = Rig::new(8, 8, [0.1; 3]);
    let mesh = rig.add(&plane(1.0, 0.0), crate::scene::Surface::Diffuse { color: [0.5; 3] }, translation(0.0, 0.0, 0.0));
    rig.instance(mesh, translation(5.0, 0.0, -2.0));
    rig.instance(mesh, translation(0.0, 0.0, 3.0));
    rig
}
fn down(p: [f32; 3], d: [f32; 3]) -> Ray {
    Ray { p, d, tmin: 0.0, tmax: 1000.0, ..Ray::default() }
}
fn trace(kg: &KernelGlobals, ray: &Ray) -> (bool, Intersection) {
    let mut isect = Intersection::default();
    let hit = scene_intersect(kg, ray, flag("PATH_RAY_VISIBILITY_CAMERA"), &mut isect);
    (hit, isect)
}
#[test]
fn a_ray_hits_the_nearest_triangle_with_its_barycentric_coordinates() {
    let kg = placed().kernel();
    // The plane at z = 3 is the nearest from above; (0.5, -0.25) is in
    // the triangle (-1,-1) (1,-1) (1,1) at u = v = 0.375.
    let (hit, isect) = trace(&kg, &down([0.5, -0.25, 8.0], [0.0, 0.0, -1.0]));
    assert!(hit);
    assert!((isect.t - 5.0).abs() < 1e-5 && (isect.u - 0.375).abs() < 1e-5 && (isect.v - 0.375).abs() < 1e-5, "{isect:?}");
    assert_eq!((isect.kind, isect.object), (flag("PRIMITIVE_TRIANGLE") as i32, 2));
}
#[test]
fn the_nearest_of_the_instances_wins_from_either_side() {
    let kg = placed().kernel();
    let (_, above) = trace(&kg, &down([0.0, 0.5, 8.0], [0.0, 0.0, -1.0]));
    let (_, below) = trace(&kg, &down([0.0, 0.5, -8.0], [0.0, 0.0, 1.0]));
    assert_eq!((above.object, below.object), (2, 0));
    assert!((above.t - 5.0).abs() < 1e-5 && (below.t - 8.0).abs() < 1e-5);
}
#[test]
fn a_ray_enters_an_instance_through_the_inverse_of_its_transform() {
    let kg = placed().kernel();
    // The instance at (5, 0, -2): the same mesh, in another place. A ray
    // over it hits it at the distance in the world.
    let (hit, isect) = trace(&kg, &down([5.5, 0.25, 1.0], [0.0, 0.0, -1.0]));
    assert!(hit && isect.object == 1 && (isect.t - 3.0).abs() < 1e-5, "{isect:?}");
    // The point it hits in object space is the same as for the original.
    let (_, home) = trace(&kg, &down([0.5, 0.25, 1.0], [0.0, 0.0, -1.0]));
    assert!(((isect.u, isect.v) == (home.u, home.v)) && isect.prim == home.prim);
}
#[test]
fn a_ray_that_misses_or_runs_short_of_the_instances_hits_nothing() {
    let kg = placed().kernel();
    let (hit, isect) = trace(&kg, &down([9.0, 9.0, 8.0], [0.0, 0.0, -1.0]));
    let none = |name: &str| crate::kernel::constant(name).unwrap() as i32;
    assert!(!hit && isect.prim == none("PRIM_NONE") && isect.object == none("OBJECT_NONE"));
    let short = Ray { tmax: 4.0, ..down([0.0, 0.5, 8.0], [0.0, 0.0, -1.0]) };
    assert!(!trace(&kg, &short).0);
    // Beyond tmin.
    let past = Ray { tmin: 5.5, ..down([0.0, 0.5, 8.0], [0.0, 0.0, -1.0]) };
    assert!(trace(&kg, &past).1.object == 0 && (trace(&kg, &past).1.t - 8.0).abs() < 1e-5);
}
#[test]
fn a_ray_does_not_hit_the_instance_s_primitive_it_left() {
    let kg = placed().kernel();
    let (_, first) = trace(&kg, &down([0.0, 0.5, 8.0], [0.0, 0.0, -1.0]));
    // From the hit point, along the same line: the same triangle is
    // skipped, and the next is the plane at the origin below it.
    let mut again = down([0.0, 0.5, 3.0], [0.0, 0.0, -1.0]);
    (again.own.object, again.own.prim) = (first.object, first.prim);
    let (hit, second) = trace(&kg, &again);
    assert!(hit && second.object == 0 && (second.t - 3.0).abs() < 1e-5);
}