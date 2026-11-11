// SPDX-License-Identifier: Apache-2.0
use crate::scene::*;

#[test]
fn the_arrays_of_cycles_device_scene_are_listed_in_its_order() {
    let array = DeviceScene::array_list();
    assert_eq!(array.len(), 53);
    assert_eq!(array[0], ("bvh_nodes", "float4", false));
    assert!(array.contains(&("objects", "KernelObject", false)));
    assert!(array.contains(&("sample_pattern_lut", "float", false)));
}

#[test]
fn only_the_image_tile_arrays_are_written_by_a_kernel() {
    let writable: Vec<&str> = DeviceScene::array_list().into_iter().filter(|(_, _, writable)| *writable).map(|(name, _, _)| name).collect();
    assert_eq!(writable, ["image_texture_tile_descriptors", "image_texture_tile_access_state"]);
}

#[test]
fn a_new_device_scene_is_empty_and_its_data_zero() {
    let dscene = DeviceScene::new();
    assert!(dscene.sample_pattern_lut.is_empty());
    assert!(dscene.object.is_empty() && dscene.attribute_map.is_empty() && dscene.tri_vert.is_empty() && dscene.object_flag.is_empty());
    assert_eq!((dscene.object.layout().name, dscene.attribute_map.layout().name), ("KernelObject", "AttributeMap"));
    assert_eq!(dscene, DeviceScene::default());
    assert!(dscene.data.word().iter().all(|word| *word == 0));
}

#[test]
fn a_new_device_scene_has_no_tree() {
    let dscene = DeviceScene::new();
    assert!(dscene.bvh_node.is_empty() && dscene.bvh_leaf_node.is_empty() && dscene.object_node.is_empty());
    assert!(dscene.camera_motion.is_empty());
    assert!(dscene.prim_type.is_empty() && dscene.prim_visibility.is_empty() && dscene.prim_index.is_empty() && dscene.prim_object.is_empty() && dscene.prim_time.is_empty());
}
