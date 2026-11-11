// SPDX-License-Identifier: Apache-2.0
use super::*;

#[test]
fn what_a_proxy_fed_is_linked_to_what_fed_the_proxy() {
    let mut graph = graph_of(&["value", "convert_float_to_float", "emission", "math"]);
    link(&mut graph, (1, "Value"), (2, "value_float"));
    link(&mut graph, (2, "value_float"), (3, "Strength"));
    link(&mut graph, (2, "value_float"), (4, "Value1"));
    graph.remove_proxy_node();
    assert_eq!(kind_list(&graph), ["output", "value", "emission", "math"]);
    assert_eq!((source(&graph, 3, "Strength"), source(&graph, 4, "Value1")), (Some((1, "Value")), Some((1, "Value"))));
}

#[test]
fn a_proxy_fed_by_nothing_hands_its_value_on() {
    let mut graph = graph_of(&["convert_float_to_float", "emission"]);
    set(&mut graph, 1, "value_float", Value::Float(0.7));
    link(&mut graph, (1, "value_float"), (2, "Strength"));
    graph.remove_proxy_node();
    assert_eq!((kind_list(&graph), source(&graph, 2, "Strength"), float(&graph, 2, "strength")), (vec!["output", "emission"], None, 0.7));
}

#[test]
fn a_convert_node_that_led_only_to_a_default_link_goes_with_it() {
    // A group's unset colour socket linked to a normal: the convert node
    // the link put between is of no use once the proxy is gone.
    let mut graph = graph_of(&["convert_color_to_color", "diffuse_bsdf"]);
    link(&mut graph, (1, "value_color"), (2, "Normal"));
    assert_eq!(kind_list(&graph), ["output", "convert_color_to_color", "diffuse_bsdf", "convert_color_to_normal"]);
    graph.remove_proxy_node();
    assert_eq!((kind_list(&graph), source(&graph, 2, "Normal")), (vec!["output", "diffuse_bsdf"], None));
}

#[test]
fn a_convert_node_that_also_feeds_a_plain_input_stays() {
    let mut graph = graph_of(&["convert_color_to_color", "math", "math"]);
    link(&mut graph, (1, "value_color"), (2, "Value1"));
    let convert = of_kind(&graph, "convert_color_to_float")[0];
    link(&mut graph, (convert, "value_float"), (3, "Value1"));
    graph.remove_proxy_node();
    assert_eq!(kind_list(&graph), ["output", "math", "math", "convert_color_to_float"]);
    assert_eq!((source(&graph, 2, "Value1"), source(&graph, convert, "value_color")), (Some((convert, "value_float")), None));
}

#[test]
fn a_graph_with_no_proxy_is_left_as_it_is() {
    let mut graph = graph_of(&["noise_texture", "math"]);
    link(&mut graph, (1, "Color"), (2, "Value1"));
    let before = graph.clone();
    graph.remove_proxy_node();
    assert_eq!(graph, before);
}
