// SPDX-License-Identifier: Apache-2.0
//! Tests of `src/scene/shader/graph`, one file for each file there. This
//! file tests `mod.rs` and holds what the others build a graph with.

mod bump;
mod clean;
mod closure;
mod hash;
mod input;
mod linear;
mod link;
mod node;
mod proxy;
mod socket;

use std::collections::BTreeSet;

use crate::graph::{SocketKind, Value};
use crate::kernel::{constant, flag};
use crate::scene::*;

/// A graph with a node of each of these types after the output: their
/// ids are 1, 2 and on.
pub(crate) fn graph_of(kind: &[&str]) -> ShaderGraph {
    let mut graph = ShaderGraph::new();
    for (at, kind) in kind.iter().enumerate() {
        assert_eq!(graph.create_node(kind), Some(at + 1), "{kind}");
    }
    graph
}

/// Link an output to an input, each given as its node and its label.
pub(crate) fn link(graph: &mut ShaderGraph, from: (usize, &str), to: (usize, &str)) {
    let from = graph.output_of(from.0, from.1).expect("the output is there");
    let to = graph.input_of(to.0, to.1).expect("the input is there");
    graph.connect(from, to);
}

/// The node and the output an input takes its value from.
pub(crate) fn source(graph: &ShaderGraph, node: usize, input: &str) -> Option<(usize, &'static str)> {
    let from = graph.link_of(graph.input_of(node, input)?)?;
    Some((from.node, graph.output_socket(from)?.name()))
}

/// The types of the nodes in the graph, in order.
pub(crate) fn kind_list(graph: &ShaderGraph) -> Vec<&'static str> {
    graph.node_list().map(ShaderNode::type_name).collect()
}

/// The ids of the nodes of a type.
pub(crate) fn of_kind(graph: &ShaderGraph, kind: &str) -> Vec<usize> {
    graph.node_list().filter(|node| node.type_name() == kind).map(|node| node.id).collect()
}

pub(crate) fn set(graph: &mut ShaderGraph, node: usize, socket: &str, value: Value) {
    assert!(graph.node_mut(node).expect("the node is there").node.set(socket, value), "{socket}");
}

pub(crate) fn float(graph: &ShaderGraph, node: usize, socket: &str) -> f32 {
    graph.node(node).expect("the node is there").node.float(socket)
}

#[test]
fn a_new_graph_is_an_output_node_alone() {
    let graph = ShaderGraph::new();
    assert_eq!((kind_list(&graph), graph.output(), graph.node_id_count()), (vec!["output"], 0, 1));
    let output = graph.node(0).expect("the output");
    assert_eq!(output.input.iter().map(ShaderInput::name).collect::<Vec<_>>(), ["Surface", "Volume", "Displacement", "Normal"]);
    assert!(output.output.is_empty() && !graph.finalized && !graph.simplified && graph.displacement_hash.is_empty());
    assert_eq!(graph, ShaderGraph::default());
}

#[test]
fn a_node_is_given_the_next_id_and_met_in_that_order() {
    let mut graph = graph_of(&["diffuse_bsdf", "emission"]);
    assert_eq!(graph.create_node("no_such_node"), None);
    assert_eq!(graph.create_node("math"), Some(3));
    assert_eq!(kind_list(&graph), ["output", "diffuse_bsdf", "emission", "math"]);
    assert_eq!((graph.node(2).map(|node| node.id), graph.node(9).is_none(), graph.node_id_count()), (Some(2), true, 4));
    assert!(graph.input_of(1, "Color").is_some() && graph.input_of(1, "No Such").is_none() && graph.output_of(1, "BSDF").is_some());
}

#[test]
fn an_input_depends_on_every_node_its_value_comes_through() {
    let mut graph = graph_of(&["diffuse_bsdf", "noise_texture", "math", "emission"]);
    link(&mut graph, (2, "Fac"), (3, "Value1"));
    link(&mut graph, (3, "Value"), (1, "Roughness"));
    link(&mut graph, (1, "BSDF"), (0, "Surface"));
    let mut found = BTreeSet::new();
    graph.find_dependency(&mut found, graph.input_of(0, "Surface").expect("the surface"));
    assert_eq!(found, BTreeSet::from([1, 2, 3]));
    // An input nothing is linked to depends on nothing.
    let mut found = BTreeSet::new();
    graph.find_dependency(&mut found, graph.input_of(0, "Volume").expect("the volume"));
    assert!(found.is_empty());
}

#[test]
fn nodes_are_copied_with_the_links_between_them() {
    let mut graph = graph_of(&["noise_texture", "math"]);
    set(&mut graph, 2, "value2", Value::Float(4.0));
    link(&mut graph, (1, "Fac"), (2, "Value1"));
    let copied = graph.copy_node(&BTreeSet::from([1, 2]));
    assert_eq!((copied.get(&1), copied.get(&2), kind_list(&graph).len()), (Some(&3), Some(&4), 5));
    assert_eq!((source(&graph, 4, "Value1"), float(&graph, 4, "value2")), (Some((3, "Fac")), 4.0));
    // The originals keep their own link.
    assert_eq!(source(&graph, 2, "Value1"), Some((1, "Fac")));
}

#[test]
fn simplifying_is_done_once_until_the_graph_changes() {
    let mut graph = graph_of(&["diffuse_bsdf"]);
    link(&mut graph, (1, "BSDF"), (0, "Surface"));
    graph.simplify(false);
    assert!(graph.simplified && !graph.finalized);
    // The closure's normal was nobody's: the geometry's is linked to it.
    assert_eq!(kind_list(&graph), ["output", "diffuse_bsdf", "geometry"]);
    assert!(graph.create_node("math").is_some() && !graph.simplified);
    graph.simplify(false);
    // The node nothing uses is dropped.
    assert_eq!(kind_list(&graph), ["output", "diffuse_bsdf", "geometry"]);
}

#[test]
fn finalizing_makes_the_graph_ready_to_compile_once() {
    let mut graph = graph_of(&["diffuse_bsdf"]);
    link(&mut graph, (1, "BSDF"), (0, "Surface"));
    graph.finalize(false, false, false);
    assert!(graph.finalized && graph.simplified);
    // The only closure has the whole weight.
    assert_eq!((float(&graph, 1, "surface_mix_weight"), source(&graph, 1, "Normal")), (1.0, Some((2, "Normal"))));
    let before = graph.clone();
    graph.finalize(false, true, true);
    assert_eq!(graph, before);
}
