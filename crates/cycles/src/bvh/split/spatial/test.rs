// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::bvh::split::test::*;

#[test]
fn the_plane_is_the_first_of_the_cuts_that_cost_least() {
    let mesh = under_two();
    let (range, reference) = reference_of(&mesh);
    let split = build_over(&mesh, |build| BvhSpatialSplit::new(build, &mut BvhSpatialStorage::new(), &range, &reference, 0.0));
    // Thirty-two slabs an eighth wide. Past the first triangle, at nine
    // eighths, each side has two triangles and the two boxes' areas add
    // up to 20 wherever the plane is, until it reaches the second.
    assert_eq!((split.dim, split.pos, split.sah), (0, 1.125, 40.0));
}

#[test]
fn a_reference_the_plane_goes_through_is_cut_when_that_costs_least() {
    let mesh = under_two();
    let (range, mut reference) = reference_of(&mesh);
    let split = BvhSpatialSplit { sah: 0.0, dim: 0, pos: 2.0 };
    let (left, right) = build_over(&mesh, |build| split.split(build, &mut BvhSpatialStorage::new(), &mut reference, &range));
    // The thin triangle is on both sides, each with its half of the box.
    assert_eq!((left.start, left.size, right.start, right.size, reference.len()), (0, 2, 2, 2, 4));
    assert_eq!(reference.iter().map(|reference| reference.prim_index).collect::<Vec<_>>(), [0, 2, 1, 2]);
    assert_eq!((left.bound.min[0], left.bound.max[0], right.bound.min[0], right.bound.max[0]), (0.0, 2.0, 2.0, 4.0));
    assert_eq!((reference[1].bound.max[0], reference[1].bound.max[1], reference[3].bound.min[0], reference[3].bound.max[1]), (2.0, 0.25, 2.0, 0.5));
}

#[test]
fn it_goes_to_one_side_whole_when_cutting_it_gains_nothing() {
    // Three squares in a row, the plane through the middle one: its two
    // triangles join the left whole, since halves on both sides would
    // make two longer boxes of three where one of four and one of two do.
    let mesh = strip(3);
    let (range, mut reference) = reference_of(&mesh);
    let split = BvhSpatialSplit { sah: 0.0, dim: 0, pos: 2.5 };
    let (left, right) = build_over(&mesh, |build| split.split(build, &mut BvhSpatialStorage::new(), &mut reference, &range));
    assert_eq!((reference.len(), left.size, right.start, right.size), (6, 4, 4, 2));
    assert_eq!((left.bound.max[0], right.bound.min[0]), (3.0, 4.0));
}

#[test]
fn references_wholly_on_one_side_keep_their_boxes() {
    let mesh = strip(2);
    let (range, mut reference) = reference_of(&mesh);
    let split = BvhSpatialSplit { sah: 0.0, dim: 0, pos: 1.5 };
    let (left, right) = build_over(&mesh, |build| split.split(build, &mut BvhSpatialStorage::new(), &mut reference, &range));
    assert_eq!((left.size, right.start, right.size), (2, 2, 2));
    assert_eq!((left.bound.max[0], right.bound.min[0]), (1.0, 2.0));
    assert!(reference[..2].iter().all(|reference| reference.prim_index < 2) && reference[2..].iter().all(|reference| reference.prim_index >= 2));
}
