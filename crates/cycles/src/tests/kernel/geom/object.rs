// SPDX-License-Identifier: Apache-2.0
use crate::kernel::geom::*;
use crate::kernel::{KernelGlobal, Ray, RecordLayout};
use crate::tests::kernel::bvh::instanced;
use crate::tests::kernel::near;
use crate::util::Transform;

#[test]
fn an_object_s_members_are_found_where_the_table_puts_them() {
    let field = ObjectField::new();
    let layout = RecordLayout::of("KernelObject").expect("data/record.tsv has KernelObject");
    let word = |name: &str| layout.field(name).expect("the member").word;
    assert_eq!((field.word_count, field.tfm, field.itfm, field.position_offset), (layout.word_count, word("tfm"), word("itfm"), word("position_offset")));
}

#[test]
fn an_object_s_two_transforms_undo_each_other() {
    let place = Transform::translate(3.0, 0.0, -1.0) * Transform::scale(2.0, 2.0, 2.0);
    let rig = instanced(&[Transform::IDENTITY, place]);
    let kg = KernelGlobal::new(&rig.dscene);
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
    let kg = KernelGlobal::new(&rig.dscene);
    let ray = Ray { p: [5.0, 2.0, 1.0], d: [0.0, 0.0, 1.0], tmax: 10.0, ..Ray::default() };
    let (p, dir, idir) = bvh_instance_push(&kg, 1, &ray);
    // Half the size, so half the speed: the direction is not made of
    // length one again, and a distance along it is the world's.
    assert!(near(&p, &[1.0, 1.0, 1.0]) && near(&[dir[2], idir[2]], &[0.5, 2.0]), "{p:?} {dir:?}");
    let (p, dir, idir) = bvh_instance_pop(&ray);
    assert_eq!((p, dir[2], idir[2]), (ray.p, 1.0, 1.0));
}
