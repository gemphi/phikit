// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::graph::*;

fn integrator() -> Node {
    Node::new("integrator").expect("in data/node.tsv")
}

#[test]
fn a_new_node_has_cycles_defaults() {
    let node = integrator();
    assert_eq!((node.int("max_bounce"), node.int("aa_samples"), node.int("sample_subset_length")), (7, 0, 1 << 24));
    assert_eq!((node.float("sample_clamp_indirect"), node.float("ao_distance")), (10.0, f32::MAX));
    assert!(node.boolean("use_light_tree") && !node.boolean("use_denoise"));
    assert!(node.float_array("custom_pixel_jitter_sample").is_empty());
    assert!(Node::new("no_such_node").is_none());
}

#[test]
fn a_node_has_the_sockets_of_its_type_and_no_other() {
    let node = integrator();
    assert!(node.has("seed") && !node.has("no_such_socket"));
    assert_eq!(node.node_type().name, "integrator");
}

#[test]
fn a_new_node_counts_as_modified_until_the_device_has_it() {
    let mut node = integrator();
    assert!(node.is_modified() && node.socket_is_modified("seed"));
    node.clear_modified();
    assert!(!node.is_modified() && !node.socket_is_modified("seed"));
}

#[test]
fn setting_a_socket_marks_that_socket_and_the_same_value_marks_nothing() {
    let mut node = integrator();
    node.clear_modified();
    assert!(!node.set("max_bounce", Value::Int(7)));
    assert!(!node.is_modified());
    assert!(node.set("max_bounce", Value::Int(12)));
    assert_eq!(node.int("max_bounce"), 12);
    assert!(node.socket_is_modified("max_bounce") && !node.socket_is_modified("min_bounce"));
}

#[test]
fn an_enum_is_set_by_the_key_of_its_choice() {
    let mut node = integrator();
    assert!(node.set_enum("sampling_pattern", "blue_noise_pure"));
    assert_eq!(node.int("sampling_pattern"), 2);
    assert!(!node.set_enum("sampling_pattern", "no_such_pattern"));
    assert_eq!(node.int("sampling_pattern"), 2);
}

#[test]
fn a_socket_or_the_whole_node_can_be_tagged() {
    let mut node = integrator();
    node.clear_modified();
    node.tag_socket_modified("ao_bounces");
    assert!(node.socket_is_modified("ao_bounces") && !node.socket_is_modified("seed"));
    node.tag_modified();
    assert!(node.socket_is_modified("seed"));
}

#[test]
fn an_object_s_sockets_are_read_by_their_types() {
    let mut node = Node::new("object").expect("in data/node.tsv");
    assert_eq!((node.uint("visibility"), node.uint64("light_set_membership"), node.string("asset_name")), (127, u64::MAX, ""));
    assert_eq!((node.float3("color"), node.float2("dupli_uv")), ([0.0; 3], [0.0; 2]));
    node.set("asset_name", Value::String("Set".to_string()));
    assert_eq!(node.string("asset_name"), "Set");
    let mut mesh = Node::new("mesh").expect("in data/node.tsv");
    assert!(mesh.int_array("triangles").is_empty() && mesh.boolean_array("smooth").is_empty());
    mesh.set("triangles", Value::IntArray(vec![0, 1, 2]));
    mesh.set("smooth", Value::BooleanArray(vec![true]));
    assert_eq!((mesh.int_array("triangles"), mesh.boolean_array("smooth")), (&[0, 1, 2][..], &[true][..]));
    // A mesh has its base type's sockets too.
    assert_eq!(mesh.uint("motion_steps"), 0);
}

#[test]
fn the_sockets_are_met_in_order_with_their_values() {
    let node = Node::new("math").expect("in data/node.tsv");
    let name: Vec<&str> = node.socket_list().map(|(socket, _)| socket.name.as_str()).collect();
    assert_eq!(name, ["math_type", "use_clamp", "value1", "value2", "value3", "value"]);
    assert!(node.socket_list().all(|(socket, value)| node.get(&socket.name) == Some(value)));
}

#[test]
fn a_value_copied_from_another_node_is_not_a_change() {
    let (mut first, mut second) = (Node::new("math").expect("in data/node.tsv"), Node::new("math").expect("in data/node.tsv"));
    assert!(second.set("value2", Value::Float(7.0)));
    first.clear_modified();
    first.copy_value("value1", &second, "value2");
    assert_eq!((first.float("value1"), first.is_modified()), (7.0, false));
}
