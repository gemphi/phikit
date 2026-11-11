// SPDX-License-Identifier: Apache-2.0
use super::*;

#[test]
fn a_row_of_boxes_is_cut_where_the_two_sides_cost_least() {
    let (range, mut reference) = row(&[0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0]);
    let binning = BvhObjectBinning::new(range, &reference);
    // Eight boxes go into four bins, three, two, two and one. Primitives
    // are counted in fours: three boxes 5 long and five boxes 9 long cost
    // 11 and twice 19; one leaf of the row, 15 long, costs twice 31.
    assert_eq!((binning.split_sah, binning.leaf_sah), (49.0, 62.0));
    let (left, right) = binning.split(&mut reference);
    assert_eq!((left.range.start, left.range.size, right.range.start, right.range.size), (0, 3, 3, 5));
    assert_eq!((left.range.bound.max[0], right.range.bound.min[0]), (5.0, 6.0));
    assert!(reference[..3].iter().all(|reference| reference.bound.max[0] <= 5.0));
    assert!(reference[3..].iter().all(|reference| reference.bound.min[0] >= 6.0));
    // Each side's centres have their own box, for the next cut.
    assert_eq!((left.range.cent_bound.min[0], left.range.cent_bound.max[0]), (1.0, 9.0));
}

#[test]
fn boxes_that_share_one_centre_are_halved_as_they_stand() {
    let (range, mut reference) = row(&[3.0; 5]);
    let binning = BvhObjectBinning::new(range, &reference);
    assert_eq!(binning.split_sah, f32::MAX);
    let (left, right) = binning.split(&mut reference);
    assert_eq!((left.range.start, left.range.size, right.range.start, right.range.size), (0, 2, 2, 3));
    assert_eq!(left.range.bound, right.range.bound);
}

#[test]
fn only_the_range_s_own_references_are_binned_and_moved() {
    let (_, mut reference) = row(&[100.0, 0.0, 2.0, 4.0, 6.0, 200.0]);
    let range = BvhRange { start: 1, ..range_of(&reference[1..5]) };
    let (left, right) = BvhObjectBinning::new(range, &reference).split(&mut reference);
    assert_eq!((left.range.start, left.range.size + right.range.size, right.range.end()), (1, 4, 5));
    assert_eq!((reference[0].bound.min[0], reference[5].bound.min[0]), (100.0, 200.0));
    assert!(right.range.bound.max[0] <= 7.0);
}

#[test]
fn a_long_range_has_no_more_bins_than_cycles_allows() {
    let x: Vec<f32> = (0..1000).map(|at| at as f32 * 2.0).collect();
    let (range, mut reference) = row(&x);
    let (left, right) = BvhObjectBinning::new(range, &reference).split(&mut reference);
    // A cut between two of thirty-two bins, each of some thirty boxes.
    assert_eq!(left.range.size + right.range.size, 1000);
    assert!(left.range.size >= 30 && right.range.size >= 30);
    assert!(left.range.bound.max[0] < right.range.bound.min[0]);
}
