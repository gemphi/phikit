// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::kernel::bvh::test::*;
use crate::kernel::constant;

#[test]
fn a_walk_starts_at_the_root_the_scene_wrote_with_cycles_numbers() {
    let rig = rig_of(vec![strip(8)]);
    let bvh = KernelGlobal::of(&rig.dscene).bvh;
    assert_eq!(bvh.root, rig.dscene.data.int("bvh", "root"));
    let number = |name: &str| constant(name).expect("in data/constant.tsv");
    assert_eq!((i64::from(bvh.sentinel), i64::from(bvh.prim_none), i64::from(bvh.object_none)), (number("ENTRYPOINT_SENTINEL"), -1, -1));
    assert_eq!((i64::from(bvh.primitive_all), i64::from(bvh.primitive_triangle)), (number("PRIMITIVE_ALL"), number("PRIMITIVE_TRIANGLE")));
    assert_eq!(i64::from(bvh.shadow_opaque), number("PATH_RAY_VISIBILITY_SHADOW_OPAQUE"));
    // The mark is no node: a tree is far smaller, and a leaf is negative.
    assert!(bvh.sentinel as usize > rig.dscene.bvh_node.len());
}
