// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::util::{BoundBox, Transform};

#[test]
fn a_triangle_is_cut_into_the_boxes_of_its_two_parts() {
    let mesh = under_two();
    let (_, reference) = reference_of(&mesh);
    let (left, right) = build_over(&mesh, |build| BvhSpatialSplit::split_reference(build, &reference[2], 0, 2.0));
    assert_eq!((left.bound.min, left.bound.max), ([0.0; 3], [2.0, 0.25, 0.0]));
    assert_eq!((right.bound.min, right.bound.max), ([2.0, 0.0, 0.0], [4.0, 0.5, 0.0]));
    assert_eq!((left.prim_index, right.prim_index, left.prim_type, right.prim_object), (2, 2, reference[2].prim_type, 0));
}

#[test]
fn a_triangle_wholly_on_one_side_leaves_nothing_on_the_other() {
    let mesh = under_two();
    let (_, reference) = reference_of(&mesh);
    let (left, right) = build_over(&mesh, |build| BvhSpatialSplit::split_reference(build, &reference[0], 0, 2.0));
    assert_eq!(left.bound, reference[0].bound);
    assert!(!right.bound.valid());
}

#[test]
fn a_part_is_never_larger_than_the_reference_it_is_cut_from() {
    // A reference that is itself a part: the left of the thin triangle.
    let mesh = under_two();
    let (_, reference) = reference_of(&mesh);
    let (left, right) = build_over(&mesh, |build| {
        let (part, _) = BvhSpatialSplit::split_reference(build, &reference[2], 0, 2.0);
        BvhSpatialSplit::split_reference(build, &part, 0, 1.0)
    });
    assert_eq!((left.bound.max[0], right.bound.min[0], right.bound.max[0], right.bound.max[1]), (1.0, 1.0, 2.0, 0.25));
}

#[test]
fn an_object_is_cut_as_its_triangles_where_its_transform_puts_them() {
    let mesh = square();
    let mut object = [Object::new(0)];
    assert!(object[0].set_transform(Transform::translate(10.0, 0.0, 0.0)));
    let reference = BvhReference::new(BoundBox { min: [10.0, 0.0, 0.0], max: [11.0, 1.0, 0.0] }, -1, 0, 0);
    let build_pack = &mut PackedBvh::default();
    let build = BvhBuild::new(&object, std::slice::from_ref(&mesh), build_pack, BvhParam::default());
    let (left, right) = BvhSpatialSplit::split_reference(&build, &reference, 0, 10.5);
    assert_eq!((left.bound.min[0], left.bound.max[0], right.bound.min[0], right.bound.max[0]), (10.0, 10.5, 10.5, 11.0));
    assert_eq!((left.prim_index, right.prim_type), (-1, 0));
}
