// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::bvh::split::test::*;

#[test]
fn a_row_is_sorted_and_cut_where_the_two_boxes_cost_least() {
    let (range, mut reference) = row(&[14.0, 4.0, 10.0, 0.0, 6.0, 12.0, 2.0, 8.0]);
    let split = BvhObjectSplit::new(&BvhParam::default(), &mut BvhSpatialStorage::new(), &range, &mut reference, 124.0);
    // Four boxes on each side, each side 7 long: an area of 30 for four.
    assert_eq!((split.dim, split.left_count, split.sah), (0, 4, 124.0 + 240.0));
    assert_eq!((split.left_bound.max[0], split.right_bound.min[0]), (7.0, 8.0));
    let (left, right) = split.split(&mut reference, &range);
    assert_eq!((left.start, left.size, right.start, right.size), (0, 4, 4, 4));
    assert_eq!((left.bound, right.bound), (split.left_bound, split.right_bound));
    assert_eq!(reference.iter().map(|reference| reference.bound.min[0]).collect::<Vec<_>>(), [0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0]);
}

#[test]
fn only_the_range_is_sorted() {
    let (_, mut reference) = row(&[50.0, 6.0, 4.0, 2.0, 0.0, 40.0]);
    let range = BvhRange { start: 1, ..range_of(&reference[1..5]) };
    let split = BvhObjectSplit::new(&BvhParam::default(), &mut BvhSpatialStorage::new(), &range, &mut reference, 0.0);
    let (left, right) = split.split(&mut reference, &range);
    assert_eq!((left.start, left.size, right.start, right.size), (1, 2, 3, 2));
    assert_eq!(reference.iter().map(|reference| reference.bound.min[0]).collect::<Vec<_>>(), [50.0, 0.0, 2.0, 4.0, 6.0, 40.0]);
}

#[test]
fn one_reference_cannot_be_cut() {
    let (range, mut reference) = row(&[0.0]);
    let split = BvhObjectSplit::new(&BvhParam::default(), &mut BvhSpatialStorage::new(), &range, &mut reference, 0.0);
    assert_eq!((split.sah, split.left_count), (f32::MAX, 0));
}
