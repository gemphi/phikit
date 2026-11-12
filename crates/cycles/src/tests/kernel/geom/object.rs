// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::kernel::geom::*;
use crate::kernel::{KernelGlobal, Ray};
use crate::tests::kernel::bvh::instanced;
use crate::tests::kernel::near;
use crate::util::Transform;

#[test]
fn a_point_and_a_normal_go_out_of_an_object_and_a_direction_into_it() {
    let place = Transform::translate(3.0, 0.0, -1.0) * Transform::scale(2.0, 2.0, 4.0);
    let rig = instanced(&[Transform::IDENTITY, place]);
    let kg = KernelGlobal::of(&rig.dscene);
    assert!(near(&object_position_transform(&kg, 1, [0.5, 0.25, 1.0]), &[4.0, 0.5, 3.0]));
    assert!(near(&object_inverse_dir_transform(&kg, 1, [2.0, 2.0, 4.0]), &[1.0, 1.0, 1.0]));
    // A normal goes by the inverse's transpose: a slope of the squashed
    // side stays at right angles to it, and of length one.
    let n = object_normal_transform(&kg, 1, crate::util::vector::normalize([1.0, 0.0, 1.0]));
    assert!(near(&n, &crate::util::vector::normalize([2.0, 0.0, 1.0])), "{n:?}");
}
#[test]
fn an_object_s_two_transforms_undo_each_other() {
    let place = Transform::translate(3.0, 0.0, -1.0) * Transform::scale(2.0, 2.0, 2.0);
    let rig = instanced(&[Transform::IDENTITY, place]);
    let kg = KernelGlobal::of(&rig.dscene);
    assert_eq!(object_fetch_transform(&kg, 1, ObjectTransform::Object), place);
    let back = object_fetch_transform(&kg, 1, ObjectTransform::Inverse);
    assert!(near(&back.point(place.point([0.5, 0.25, 4.0])), &[0.5, 0.25, 4.0]));
    assert_eq!(object_fetch_transform(&kg, 0, ObjectTransform::Inverse), Transform::IDENTITY);
}

#[test]
fn a_direction_is_kept_off_zero_so_that_its_inverse_is_a_number() {
    let dir = bvh_clamp_direction([0.0, -0.0, 0.5]);
    assert!(dir[0] > 0.0 && dir[1] < 0.0 && dir[2] == 0.5, "{dir:?}");
    let idir = bvh_inverse_direction(dir);
    assert!(idir.iter().all(|part| part.is_finite()) && idir[1] < 0.0 && idir[2] == 2.0, "{idir:?}");
}

#[test]
fn inside_an_instance_the_ray_is_as_the_object_sees_it() {
    let place = Transform::translate(3.0, 0.0, -1.0) * Transform::scale(2.0, 2.0, 2.0);
    let rig = instanced(&[Transform::IDENTITY, place]);
    let kg = KernelGlobal::of(&rig.dscene);
    let ray = Ray { p: [5.0, 2.0, 1.0], d: [0.0, 0.0, 1.0], tmax: 10.0, ..Ray::default() };
    let (p, dir, idir) = bvh_instance_push(&kg, 1, &ray);
    // Half the size, so half the speed: the direction is not made of
    // length one again, and a distance along it is the world's.
    assert!(near(&p, &[1.0, 1.0, 1.0]) && near(&[dir[2], idir[2]], &[0.5, 2.0]), "{p:?} {dir:?}");
    let (p, dir, idir) = bvh_instance_pop(&ray);
    assert_eq!((p, dir[2], idir[2]), (ray.p, 1.0, 1.0));
}
