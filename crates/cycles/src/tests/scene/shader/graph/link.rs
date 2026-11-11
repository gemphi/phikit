// SPDX-License-Identifier: Apache-2.0
use super::*;

#[test]
fn sockets_of_one_type_are_linked_directly() {
    let mut graph = graph_of(&["noise_texture", "math"]);
    link(&mut graph, (1, "Fac"), (2, "Value1"));
    let (from, to) = (graph.output_of(1, "Fac").expect("the output"), graph.input_of(2, "Value1").expect("the input"));
    assert_eq!((graph.link_of(to), graph.linked(from)), (Some(from), vec![to]));
    assert_eq!(kind_list(&graph).len(), 3);
}

#[test]
fn an_input_that_is_linked_already_keeps_its_link() {
    let mut graph = graph_of(&["noise_texture", "math", "math"]);
    link(&mut graph, (1, "Fac"), (3, "Value1"));
    link(&mut graph, (2, "Value"), (3, "Value1"));
    assert_eq!(source(&graph, 3, "Value1"), Some((1, "Fac")));
    assert!(graph.linked(graph.output_of(2, "Value").expect("the output")).is_empty());
    // One output may feed many inputs.
    link(&mut graph, (1, "Fac"), (3, "Value2"));
    assert_eq!(graph.linked(graph.output_of(1, "Fac").expect("the output")).len(), 2);
}

#[test]
fn a_convert_node_goes_between_two_types() {
    let mut graph = graph_of(&["noise_texture", "math"]);
    link(&mut graph, (1, "Color"), (2, "Value1"));
    assert_eq!(kind_list(&graph), ["output", "noise_texture", "math", "convert_color_to_float"]);
    assert_eq!(graph.node(3).map(|node| node.special_type), Some(ShaderNodeSpecialType::Autoconvert));
    assert_eq!((source(&graph, 3, "value_color"), source(&graph, 2, "Value1")), (Some((1, "Color")), Some((3, "value_float"))));
}

#[test]
fn a_colour_or_a_number_becomes_a_closure_through_an_emission_node() {
    let mut graph = graph_of(&["noise_texture", "value"]);
    link(&mut graph, (1, "Color"), (0, "Surface"));
    let emission = graph.node(3).expect("made by the link");
    assert_eq!((emission.type_name(), emission.from_auto_conversion, emission.node.float3("color"), emission.node.float("strength")), ("emission", true, [1.0; 3], 1.0));
    assert_eq!((source(&graph, 3, "Color"), source(&graph, 0, "Surface")), (Some((1, "Color")), Some((3, "Emission"))));
    // A number goes to the strength.
    link(&mut graph, (2, "Value"), (0, "Volume"));
    assert_eq!((source(&graph, 4, "Strength"), source(&graph, 0, "Volume")), (Some((2, "Value")), Some((4, "Emission"))));
}

#[test]
fn a_closure_converts_to_nothing() {
    let mut graph = graph_of(&["diffuse_bsdf", "math"]);
    link(&mut graph, (1, "BSDF"), (2, "Value1"));
    assert_eq!((source(&graph, 2, "Value1"), kind_list(&graph).len()), (None, 3));
}

#[test]
fn unlinking_an_input_or_an_output_is_seen_from_both_ends() {
    let mut graph = graph_of(&["noise_texture", "math"]);
    link(&mut graph, (1, "Fac"), (2, "Value1"));
    link(&mut graph, (1, "Fac"), (2, "Value2"));
    graph.simplified = true;
    let (fac, first) = (graph.output_of(1, "Fac").expect("the output"), graph.input_of(2, "Value1").expect("the input"));
    graph.disconnect_input(first);
    assert_eq!((graph.link_of(first), graph.linked(fac).len(), graph.simplified), (None, 1, false));
    graph.disconnect_output(fac);
    assert!(graph.linked(fac).is_empty() && source(&graph, 2, "Value2").is_none());
}

#[test]
fn an_input_takes_over_another_s_link_and_value() {
    let mut graph = graph_of(&["noise_texture", "math", "math"]);
    link(&mut graph, (1, "Fac"), (2, "Value1"));
    set(&mut graph, 2, "value2", Value::Float(0.75));
    let input = |graph: &ShaderGraph, node: usize, name: &str| graph.input_of(node, name).expect("the input");
    graph.relink_input(input(&graph, 2, "Value1"), input(&graph, 3, "Value1"));
    assert_eq!((source(&graph, 2, "Value1"), source(&graph, 3, "Value1")), (None, Some((1, "Fac"))));
    // One with no link hands only its value on.
    graph.relink_input(input(&graph, 2, "Value2"), input(&graph, 3, "Value2"));
    assert_eq!((float(&graph, 3, "value2"), source(&graph, 3, "Value2")), (0.75, None));
}

#[test]
fn what_an_output_fed_is_moved_to_another_or_left_unlinked() {
    let mut graph = graph_of(&["noise_texture", "noise_texture", "math"]);
    link(&mut graph, (1, "Fac"), (3, "Value1"));
    link(&mut graph, (1, "Fac"), (3, "Value2"));
    let fac = |node: usize| OutputId { node, socket: 0 };
    graph.relink_output(fac(1), Some(fac(2)));
    assert_eq!((source(&graph, 3, "Value1"), source(&graph, 3, "Value2"), graph.linked(fac(1)).len()), (Some((2, "Fac")), Some((2, "Fac")), 0));
    graph.relink_output(fac(2), None);
    assert_eq!((source(&graph, 3, "Value1"), graph.linked(fac(2)).len()), (None, 0));
}

#[test]
fn a_node_is_bypassed_by_linking_around_it() {
    let mut graph = graph_of(&["noise_texture", "math", "emission"]);
    link(&mut graph, (1, "Fac"), (2, "Value1"));
    link(&mut graph, (2, "Value"), (3, "Strength"));
    let (value, fac) = (graph.output_of(2, "Value").expect("the output"), graph.output_of(1, "Fac").expect("the output"));
    graph.relink_node(2, value, Some(fac));
    assert_eq!((source(&graph, 3, "Strength"), source(&graph, 2, "Value1")), (Some((1, "Fac")), None));
    assert!(graph.linked(value).is_empty());
}
