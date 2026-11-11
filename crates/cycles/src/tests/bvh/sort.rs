// SPDX-License-Identifier: Apache-2.0
use std::cmp::Ordering;

use super::*;

#[test]
fn references_are_put_in_order_of_their_centres_along_the_axis() {
    let (_, mut reference) = row(&[6.0, 0.0, 4.0, 2.0]);
    bvh_reference_sort(&mut reference, 0);
    assert_eq!(reference.iter().map(|reference| reference.bound.min[0]).collect::<Vec<_>>(), [0.0, 2.0, 4.0, 6.0]);
    // Along y they all have one centre: the primitives' order decides.
    bvh_reference_sort(&mut reference, 1);
    assert_eq!(reference.iter().map(|reference| reference.prim_index).collect::<Vec<_>>(), [0, 1, 2, 3]);
}

#[test]
fn equal_centres_are_told_apart_by_object_then_primitive_then_type() {
    let (_, reference) = row(&[0.0]);
    let base = reference[0];
    let compare = |a: BvhReference, b: BvhReference| bvh_reference_compare(0, &a, &b);
    assert_eq!(compare(base, base), Ordering::Equal);
    assert_eq!(compare(base, BvhReference { prim_object: 1, prim_index: -1, ..base }), Ordering::Less);
    assert_eq!(compare(BvhReference { prim_index: 3, ..base }, base), Ordering::Greater);
    assert_eq!(compare(base, BvhReference { prim_type: base.prim_type + 64, ..base }), Ordering::Less);
}
