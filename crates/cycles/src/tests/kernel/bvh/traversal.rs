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
