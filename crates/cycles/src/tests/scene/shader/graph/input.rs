// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;

#[test]
fn a_closure_s_normal_is_the_geometry_s_and_one_node_serves_all() {
    let mut graph = graph_of(&["diffuse_bsdf", "glossy_bsdf"]);
    graph.default_input(false);
    assert_eq!(kind_list(&graph), ["output", "diffuse_bsdf", "glossy_bsdf", "geometry", "convert_normal_to_vector"]);
    assert_eq!((source(&graph, 1, "Normal"), source(&graph, 2, "Normal")), (Some((3, "Normal")), Some((3, "Normal"))));
    // A glossy closure's tangent too, a vector, through a convert node.
    assert_eq!((source(&graph, 2, "Tangent"), source(&graph, 4, "value_normal")), (Some((4, "value_vector")), Some((3, "Tangent"))));
}

#[test]
fn a_texture_s_coordinate_is_the_generated_one_and_an_image_s_the_uv() {
    let mut graph = graph_of(&["noise_texture", "image_texture"]);
    graph.default_input(false);
    assert_eq!(kind_list(&graph), ["output", "noise_texture", "image_texture", "texture_coordinate"]);
    assert_eq!((source(&graph, 1, "Vector"), source(&graph, 2, "Vector")), (Some((3, "Generated")), Some((3, "UV"))));
}

#[test]
fn an_input_that_is_linked_is_left_alone() {
    let mut graph = graph_of(&["diffuse_bsdf", "bump"]);
    link(&mut graph, (2, "Normal"), (1, "Normal"));
    graph.default_input(false);
    assert_eq!(source(&graph, 1, "Normal"), Some((2, "Normal")));
    // The bump node's own normal is the geometry's.
    assert_eq!(source(&graph, 2, "Normal"), Some((3, "Normal")));
}

#[test]
fn an_ies_light_looks_along_the_incoming_direction_in_the_object_s_space() {
    let mut graph = graph_of(&["ies_light"]);
    graph.default_input(false);
    assert_eq!(kind_list(&graph), ["output", "ies_light", "geometry", "vector_transform", "convert_vector_to_point"]);
    assert_eq!((source(&graph, 1, "Vector"), source(&graph, 4, "value_vector"), source(&graph, 3, "Vector")), (Some((4, "value_point")), Some((3, "Vector")), Some((2, "Incoming"))));
    let transform = &graph.node(3).expect("made").node;
    let number = |name: &str| constant(name).expect("in data/constant.tsv") as i32;
    assert_eq!(transform.int("transform_type"), number("NODE_VECTOR_TRANSFORM_TYPE_NORMAL"));
    assert_eq!((transform.int("convert_from"), transform.int("convert_to")), (number("NODE_VECTOR_TRANSFORM_CONVERT_SPACE_WORLD"), number("NODE_VECTOR_TRANSFORM_CONVERT_SPACE_OBJECT")));
}

#[test]
fn an_input_only_osl_has_is_linked_only_for_osl() {
    let mut graph = graph_of(&["fresnel"]);
    graph.default_input(false);
    assert_eq!((source(&graph, 1, "Normal"), kind_list(&graph).len()), (None, 2));
    graph.default_input(true);
    assert_eq!(source(&graph, 1, "Normal"), Some((2, "Normal")));
}

#[test]
fn nodes_with_nothing_to_ask_for_add_nothing() {
    let mut graph = graph_of(&["math", "emission"]);
    let before = graph.clone();
    graph.default_input(true);
    assert_eq!(graph, before);
}
