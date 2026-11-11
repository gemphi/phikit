// SPDX-License-Identifier: Apache-2.0
use crate::util::*;

#[test]
fn a_default_is_found_by_its_class_and_member() {
    assert_eq!(param_default("BVHParams", "max_triangle_leaf_size"), 8.0);
    assert_eq!(param_default("SceneParams", "hair_subdivisions"), 3.0);
    // Two classes may have a member of one name.
    assert_eq!((param_default("BVHParams", "bvh_layout"), param_default("SceneParams", "bvh_layout")), (1.0, 2.0));
}

#[test]
fn true_is_one_and_a_fraction_is_kept() {
    assert_eq!((param_default("BVHParams", "use_spatial_split"), param_default("BVHParams", "top_level")), (1.0, 0.0));
    assert_eq!(param_default("BVHParams", "spatial_split_alpha") as f32, 1e-5);
}
