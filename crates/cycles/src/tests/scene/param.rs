// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::bvh::{BvhLayout, BvhType};
use crate::scene::*;
use crate::session::ShadingSystem;

#[test]
fn a_scene_starts_as_cycles_has_it() {
    let param = SceneParam::default();
    assert_eq!((param.shading_system, param.bvh_layout, param.bvh_type), (ShadingSystem::Svm, BvhLayout::of("AUTO"), BvhType::Dynamic));
    assert!(!param.use_bvh_spatial_split && param.use_bvh_compact_structure && param.use_bvh_unaligned_node);
    assert_eq!((param.bvh_time_step_count, param.hair_subdivision, param.hair_shape, param.texture_limit), (0, 3, 0, 0));
    assert!(param.use_texture_cache && !param.auto_texture_cache && param.texture_cache_path.is_empty() && param.background);
    assert_eq!(param.texture_resolution, 1.0);
}

#[test]
fn a_curve_is_cut_into_two_pieces_for_each_subdivision_up_to_sixteen() {
    let cut = |hair_subdivision: i32| SceneParam { hair_subdivision, ..SceneParam::default() }.curve_subdivision();
    assert_eq!([0, 1, 3, 4, 9].map(cut), [1, 2, 8, 16, 16]);
}

#[test]
fn a_change_makes_the_scene_anew_but_for_where_it_renders() {
    let param = SceneParam::default();
    assert!(!param.modified(&param.clone()));
    assert!(!param.modified(&SceneParam { background: false, ..param.clone() }));
    assert!(param.modified(&SceneParam { bvh_type: BvhType::Static, ..param.clone() }));
    assert!(param.modified(&SceneParam { bvh_time_step_count: 2, ..param.clone() }));
    assert!(param.modified(&SceneParam { texture_cache_path: "cache".to_string(), ..param.clone() }));
}
