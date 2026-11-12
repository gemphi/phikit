// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::shader::graph::test::*;

fn bump_of(graph: &ShaderGraph, node: usize) -> (ShaderBump, f32) {
    let node = graph.node(node).expect("the node is there");
    (node.bump, node.bump_filter_width)
}

#[test]
fn what_a_bump_s_height_depends_on_is_there_three_times() {
    // A texture through a math node into the height.
    let mut graph = graph_of(&["noise_texture", "math", "bump"]);
    link(&mut graph, (1, "Fac"), (2, "Value1"));
    link(&mut graph, (2, "Value"), (3, "Height"));
    graph.refine_bump_node();
    assert_eq!(kind_list(&graph), ["output", "noise_texture", "math", "bump", "noise_texture", "math", "noise_texture", "math"]);
    // The originals are the centre; the copies one step along x and y.
    assert_eq!((source(&graph, 3, "Height"), source(&graph, 3, "SampleCenter")), (None, Some((2, "Value"))));
    assert_eq!((source(&graph, 3, "SampleX"), source(&graph, 3, "SampleY")), (Some((5, "Value")), Some((7, "Value"))));
    assert_eq!((source(&graph, 5, "Value1"), source(&graph, 7, "Value1")), (Some((4, "Fac")), Some((6, "Fac"))));
    assert_eq!([1, 2, 4, 5, 6, 7].map(|node| bump_of(&graph, node).0), [ShaderBump::Center, ShaderBump::Center, ShaderBump::Dx, ShaderBump::Dx, ShaderBump::Dy, ShaderBump::Dy]);
    // Each is told the bump node's filter width; the bump node is no sample.
    assert_eq!((bump_of(&graph, 4), bump_of(&graph, 3)), ((ShaderBump::Dx, 0.1), (ShaderBump::None, 0.0)));
}

#[test]
fn a_bump_with_no_height_linked_is_left_alone() {
    let mut graph = graph_of(&["bump", "noise_texture"]);
    let before = graph.clone();
    graph.refine_bump_node();
    assert_eq!(graph, before);
}

#[test]
fn a_displacement_bends_the_normal_through_three_heights() {
    let mut graph = graph_of(&["noise_texture", "displacement"]);
    link(&mut graph, (1, "Fac"), (2, "Height"));
    link(&mut graph, (2, "Displacement"), (0, "Displacement"));
    graph.bump_from_displacement(true);
    // The bump node, three copies of the two nodes, a node to set the
    // normal, a dot product for each copy, and the geometry.
    let made = ["bump", "noise_texture", "displacement", "noise_texture", "displacement", "noise_texture", "displacement", "set_normal", "vector_math", "vector_math", "vector_math", "geometry"];
    assert_eq!(kind_list(&graph)[3..15], made);
    // A normal goes into a vector input through a convert node: three
    // for the dot products and one for the node that sets the normal.
    assert_eq!(of_kind(&graph, "convert_normal_to_vector"), [15, 16, 17, 18]);
    let bump = graph.node(3).expect("made");
    assert!(bump.node.boolean("use_object_space") && bump.node.float("distance") == 1.0);
    assert_eq!((source(&graph, 0, "Normal"), source(&graph, 10, "Direction"), source(&graph, 18, "value_normal")), (Some((10, "Normal")), Some((18, "value_vector")), Some((3, "Normal"))));
    // Each height is a copy's displacement along the geometry's normal.
    assert_eq!((source(&graph, 3, "SampleCenter"), source(&graph, 3, "SampleX"), source(&graph, 3, "SampleY")), (Some((11, "Value")), Some((12, "Value")), Some((13, "Value"))));
    assert_eq!((source(&graph, 11, "Vector1"), source(&graph, 12, "Vector1"), source(&graph, 13, "Vector1")), (Some((5, "Displacement")), Some((7, "Displacement")), Some((9, "Displacement"))));
    assert_eq!((source(&graph, 11, "Vector2"), source(&graph, 15, "value_normal"), source(&graph, 3, "Normal")), (Some((15, "value_vector")), Some((14, "Normal")), Some((14, "Normal"))));
    assert_eq!(graph.node(11).map(|node| node.node.int("math_type")), constant("NODE_VECTOR_MATH_DOT_PRODUCT").map(|number| number as i32));
    // The copies are the samples; what displaces the mesh is untouched.
    assert_eq!([4, 6, 8, 2].map(|node| bump_of(&graph, node).0), [ShaderBump::Center, ShaderBump::Dx, ShaderBump::Dy, ShaderBump::None]);
    assert_eq!(source(&graph, 0, "Displacement"), Some((2, "Displacement")));
}

#[test]
fn with_no_displacement_there_is_no_bump_from_it() {
    let mut graph = graph_of(&["noise_texture", "displacement"]);
    link(&mut graph, (1, "Fac"), (2, "Height"));
    let before = graph.clone();
    graph.bump_from_displacement(false);
    assert_eq!(graph, before);
}
