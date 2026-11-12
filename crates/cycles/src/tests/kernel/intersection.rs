// SPDX-License-Identifier: Apache-2.0
use crate::kernel::*;

#[test]
fn a_new_intersection_is_all_zero_until_a_walk_fills_it() {
    let isect = Intersection::default();
    assert_eq!((isect.t, isect.u, isect.v, isect.prim, isect.object, isect.kind), (0.0, 0.0, 0.0, 0, 0, 0));
}
