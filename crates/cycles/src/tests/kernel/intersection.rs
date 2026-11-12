// SPDX-License-Identifier: Apache-2.0
use crate::kernel::*;
#[test]
fn a_new_intersection_hits_nothing_until_a_walk_fills_it() {
    let isect = Intersection::default();
    let none = |name: &str| constant(name).expect("in data/constant.tsv") as i32;
    assert_eq!((isect.t, isect.u, isect.v, isect.kind), (0.0, 0.0, 0.0, 0));
    assert_eq!((isect.prim, isect.object), (none("PRIM_NONE"), none("OBJECT_NONE")));
}
