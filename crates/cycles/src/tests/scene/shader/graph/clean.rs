// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;

/// Two maths that add one and two, each feeding one input of a third,
/// which is the strength of an emission on the surface.
fn twins() -> ShaderGraph {
    let mut graph = graph_of(&["math", "math", "math", "emission"]);
    for node in [1, 2] {
        set(&mut graph, node, "value1", Value::Float(1.0));
        set(&mut graph, node, "value2", Value::Float(2.0));
    }
    link(&mut graph, (1, "Value"), (3, "Value1"));
    link(&mut graph, (2, "Value"), (3, "Value2"));
    link(&mut graph, (3, "Value"), (4, "Strength"));
    link(&mut graph, (4, "Emission"), (0, "Surface"));
    graph
}

#[test]
fn a_node_equal_to_an_earlier_one_gives_way_to_it() {
    let mut graph = twins();
    graph.deduplicate_node();
    assert_eq!((source(&graph, 3, "Value1"), source(&graph, 3, "Value2")), (Some((1, "Value")), Some((1, "Value"))));
    // It is still in the graph, feeding nothing, until the clean-up.
    assert_eq!(kind_list(&graph).len(), 5);
    graph.clean();
    assert_eq!(kind_list(&graph), ["output", "math", "math", "emission"]);
    assert!(graph.node(2).is_none());
}

#[test]
fn nodes_that_differ_are_both_kept() {
    let mut graph = twins();
    set(&mut graph, 2, "value2", Value::Float(5.0));
    graph.clean();
    assert_eq!((source(&graph, 3, "Value2"), kind_list(&graph).len()), (Some((2, "Value")), 5));
}

#[test]
fn what_feeds_two_equal_nodes_is_merged_before_they_are_looked_at() {
    // Each twin takes its first value from a noise texture of its own.
    let mut graph = twins();
    let noise = [graph.create_node("noise_texture"), graph.create_node("noise_texture")].map(|node| node.expect("made"));
    link(&mut graph, (noise[0], "Fac"), (1, "Value1"));
    link(&mut graph, (noise[1], "Fac"), (2, "Value1"));
    graph.clean();
    // One noise texture and one of the twins are left.
    assert_eq!(kind_list(&graph), ["output", "math", "math", "emission", "noise_texture"]);
    assert_eq!(source(&graph, 3, "Value2"), Some((1, "Value")));
}

#[test]
fn a_volume_of_nothing_a_volume_is_made_of_is_unlinked() {
    let mut graph = graph_of(&["diffuse_bsdf", "scatter_volume"]);
    link(&mut graph, (1, "BSDF"), (0, "Volume"));
    graph.optimize_volume_output();
    assert_eq!(source(&graph, 0, "Volume"), None);
    link(&mut graph, (2, "Volume"), (0, "Volume"));
    graph.optimize_volume_output();
    assert_eq!(source(&graph, 0, "Volume"), Some((2, "Volume")));
}

#[test]
fn an_attribute_under_something_nonlinear_is_not_sampled_at_random() {
    let build = |math_type: &str| {
        let mut graph = graph_of(&["scatter_volume", "math", "attribute"]);
        set(&mut graph, 2, "math_type", Value::Enum(constant(math_type).expect("in data/constant.tsv") as i32));
        link(&mut graph, (3, "Fac"), (2, "Value1"));
        link(&mut graph, (2, "Value"), (1, "Density"));
        link(&mut graph, (1, "Volume"), (0, "Volume"));
        graph.optimize_volume_output();
        graph.node(3).expect("the attribute").stochastic_sample
    };
    assert!(build("NODE_MATH_MULTIPLY") && !build("NODE_MATH_POWER"));
}

#[test]
fn a_cycle_is_broken_and_the_rest_kept() {
    let mut graph = graph_of(&["math", "math", "emission"]);
    link(&mut graph, (1, "Value"), (2, "Value1"));
    link(&mut graph, (2, "Value"), (1, "Value1"));
    link(&mut graph, (2, "Value"), (3, "Strength"));
    link(&mut graph, (3, "Emission"), (0, "Surface"));
    graph.clean();
    let linked = [source(&graph, 1, "Value1"), source(&graph, 2, "Value1")];
    assert_eq!(linked.iter().flatten().count(), 1, "{linked:?}");
    assert_eq!(source(&graph, 3, "Strength"), Some((2, "Value")));
}

#[test]
fn nodes_the_output_does_not_depend_on_are_dropped_with_their_links() {
    let mut graph = graph_of(&["noise_texture", "math", "emission", "math"]);
    link(&mut graph, (1, "Fac"), (2, "Value1"));
    link(&mut graph, (1, "Fac"), (4, "Value1"));
    link(&mut graph, (4, "Value"), (3, "Strength"));
    link(&mut graph, (3, "Emission"), (0, "Surface"));
    graph.clean();
    assert_eq!(kind_list(&graph), ["output", "noise_texture", "emission", "math"]);
    // The texture no longer feeds the node that is gone.
    assert_eq!(graph.linked(graph.output_of(1, "Fac").expect("the output")), [graph.input_of(4, "Value1").expect("the input")]);
}

#[test]
fn what_an_aov_output_depends_on_is_kept_too() {
    let mut graph = graph_of(&["aov_output", "color", "value"]);
    link(&mut graph, (2, "Color"), (1, "Color"));
    graph.clean();
    assert_eq!(kind_list(&graph), ["output", "aov_output", "color"]);
}
