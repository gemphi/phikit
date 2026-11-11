// SPDX-License-Identifier: Apache-2.0
use crate::bvh::*;

fn mask(short: &[&str]) -> i32 {
    short.iter().fold(0, |mask, short| mask | BvhLayout::of(short).0)
}

#[test]
fn the_best_layout_is_the_one_asked_for_when_the_device_has_it() {
    assert_eq!(BvhLayout::best(BvhLayout::of("BVH2"), mask(&["BVH2", "EMBREE"])), BvhLayout::of("BVH2"));
    assert_eq!(BvhLayout::best(BvhLayout::of("EMBREE"), mask(&["BVH2", "EMBREE"])), BvhLayout::of("EMBREE"));
}

#[test]
fn else_it_is_the_widest_below_it_or_the_widest_there_is() {
    // `BVH_LAYOUT_AUTO` is Embree's; a device without it walks Cycles' own.
    assert_eq!(BvhLayout::of("AUTO"), BvhLayout::of("EMBREE"));
    assert_eq!(BvhLayout::best(BvhLayout::of("AUTO"), mask(&["BVH2"])), BvhLayout::of("BVH2"));
    assert_eq!(BvhLayout::best(BvhLayout::of("OPTIX"), mask(&["BVH2", "EMBREE"])), BvhLayout::of("EMBREE"));
    assert_eq!(BvhLayout::best(BvhLayout::of("BVH2"), mask(&["EMBREE", "OPTIX"])), BvhLayout::of("OPTIX"));
    assert_eq!(BvhLayout::best(BvhLayout::of("BVH2"), 0), BvhLayout::NONE);
}

#[test]
fn only_the_layouts_of_gpu_libraries_keep_instances_alone_at_the_top() {
    assert!(["OPTIX", "METAL", "HIPRT", "EMBREEGPU", "MULTI_METAL_EMBREE"].iter().all(|short| BvhLayout::of(short).instance_only()));
    assert!(!BvhLayout::of("BVH2").instance_only() && !BvhLayout::of("EMBREE").instance_only() && !BvhLayout::NONE.instance_only());
}

#[test]
fn a_type_is_static_only_by_cycles_number_for_it() {
    assert_eq!((BvhType::of(0), BvhType::of(1), BvhType::default()), (BvhType::Dynamic, BvhType::Static, BvhType::Dynamic));
}

#[test]
fn the_limits_are_those_of_cycles() {
    let limit = BvhLimit::get();
    assert_eq!((limit.max_depth, limit.max_spatial_depth, limit.spatial_bin_count), (64, 48, 32));
    assert_eq!((limit.max_bin, limit.log_block_size, limit.node_size, limit.leaf_node_size), (32, 2, 4, 1));
}

#[test]
fn the_settings_start_as_cycles_has_them() {
    let param = BvhParam::default();
    assert!(param.use_spatial_split && !param.top_level && !param.use_unaligned_node && !param.use_compact_structure);
    assert_eq!((param.spatial_split_alpha, param.unaligned_split_threshold, param.sah_node_cost, param.sah_primitive_cost), (1e-5, 0.7, 1.0, 1.0));
    assert_eq!((param.min_leaf_size, param.max_triangle_leaf_size, param.max_motion_triangle_leaf_size), (1, 8, 8));
    assert_eq!((param.max_curve_leaf_size, param.max_motion_curve_leaf_size, param.max_point_leaf_size, param.max_motion_point_leaf_size), (1, 4, 8, 8));
    assert_eq!((param.bvh_layout, param.bvh_type, param.curve_subdivision), (BvhLayout::of("BVH2"), BvhType::Dynamic, 4));
    assert!(!param.use_motion_step());
}

#[test]
fn a_node_costs_its_children_and_its_primitives() {
    let param = BvhParam { sah_node_cost: 2.0, sah_primitive_cost: 3.0, ..BvhParam::default() };
    assert_eq!((param.node_cost(2), param.primitive_cost(5), param.cost(2, 5)), (4.0, 15.0, 19.0));
}

#[test]
fn a_range_is_a_leaf_when_it_is_one_primitive_or_too_deep() {
    let param = BvhParam::default();
    assert!(param.small_enough_for_leaf(1, 0) && param.small_enough_for_leaf(0, 0) && !param.small_enough_for_leaf(2, 63));
    assert!(param.small_enough_for_leaf(1000, 64));
}

#[test]
fn time_steps_for_any_kind_of_primitive_make_leaves_hold_for_a_span() {
    assert!(BvhParam { motion_triangle_step_count: 1, ..BvhParam::default() }.use_motion_step());
    assert!(BvhParam { motion_point_step_count: 2, ..BvhParam::default() }.use_motion_step());
}
