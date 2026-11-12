// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::kernel::*;
use crate::scene::PassType;

#[test]
fn an_enumerator_has_the_number_cycles_gives_it() {
    assert_eq!(constant("SAMPLING_PATTERN_BLUE_NOISE_ROUND"), Some(4));
    assert_eq!(constant("PATH_RAY_VISIBILITY_ALL"), Some(127));
    assert_eq!(constant("NO_SUCH_CONSTANT"), None);
}

#[test]
fn a_define_has_its_number_too_also_one_that_stands_on_another() {
    assert_eq!(constant("MAX_TAB_SOBOL_SAMPLES"), Some(8192));
    assert_eq!(constant("KERNEL_FEATURE_AO_ADDITIVE"), Some(1 << 23));
    let pass = constant("KERNEL_FEATURE_AO_PASS").expect("in data/constant.tsv");
    assert_eq!(constant("KERNEL_FEATURE_AO"), Some(pass | 1 << 23));
}

#[test]
fn a_flag_is_the_constant_as_bits() {
    assert_eq!(flag("FILTER_CLOSURE_DIRECT_LIGHT"), 32);
    assert_eq!(flag("KERNEL_FEATURE_LIGHT_TREE"), 1 << 31);
}

#[test]
fn the_pass_table_and_the_constant_table_agree() {
    for name in ["PASS_COMBINED", "PASS_DEPTH", "PASS_SHADOW_CATCHER_MATTE"] {
        assert_eq!(PassType::named(name).map(|pass| i64::from(pass.0)), constant(name), "{name}");
    }
}

#[test]
fn a_manager_s_update_flags_are_named_with_its_class() {
    // Two managers use one name for different bits.
    assert_eq!((constant("ObjectManager::TRANSFORM_MODIFIED"), constant("GeometryManager::TRANSFORM_MODIFIED")), (Some(128), Some(1024)));
    assert_eq!((flag("ObjectManager::UPDATE_ALL"), flag("GeometryManager::UPDATE_NONE")), (u32::MAX, 0));
    assert_eq!(constant("TRANSFORM_MODIFIED"), None);
}

#[test]
fn a_define_of_every_bit_set_is_minus_one() {
    assert_eq!(constant("LIGHT_LINK_MASK_ALL").map(|all| all as u64), Some(u64::MAX));
    assert_eq!((constant("NODE_ATTR_FLOAT3"), constant("NODE_ATTR_MATRIX")), (Some(2), Some(5)));
}

#[test]
fn the_bvh_s_limits_are_under_their_class_and_its_node_sizes_under_their_own_names() {
    assert_eq!((constant("BVHParams::MAX_DEPTH"), constant("BVHObjectBinning::MAX_BINS"), constant("BVH_NODE_SIZE")), (Some(64), Some(32), Some(4)));
    assert_eq!((constant("BVH_TYPE_DYNAMIC"), constant("BVH_TYPE_STATIC")), (Some(0), Some(1)));
}

#[test]
fn an_enumerator_written_over_several_lines_has_its_number() {
    // Every kind of primitive, and motion.
    assert_eq!(constant("PRIMITIVE_ALL"), Some(127));
}

#[test]
fn a_socket_s_flags_are_under_their_class_and_the_svm_enums_under_their_own_names() {
    assert_eq!((constant("SocketType::LINKABLE"), constant("SocketType::LINK_NORMAL"), constant("SocketType::DEFAULT_LINK_MASK")), (Some(1), Some(512), Some(8176)));
    assert_eq!((constant("NODE_MATH_ADD"), constant("NODE_VECTOR_MATH_DOT_PRODUCT"), constant("CLOSURE_BSDF_DIFFUSE_ID")), (Some(0), Some(7), Some(2)));
}
