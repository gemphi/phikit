// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::shader::graph::test::*;

/// Walk the closures under the surface.
fn transform(graph: &mut ShaderGraph, volume: bool) {
    let name = ["Surface", "Volume"][usize::from(volume)];
    let from = graph.link_of(graph.input_of(0, name).expect("the input")).expect("something is linked");
    graph.transform_multi_closure(from.node, None, volume);
}

#[test]
fn a_closure_alone_has_the_whole_weight() {
    let mut graph = graph_of(&["diffuse_bsdf"]);
    link(&mut graph, (1, "BSDF"), (0, "Surface"));
    transform(&mut graph, false);
    assert_eq!((float(&graph, 1, "surface_mix_weight"), kind_list(&graph).len()), (1.0, 2));
}

#[test]
fn a_mix_splits_the_weight_by_its_factor() {
    let mut graph = graph_of(&["mix_closure", "diffuse_bsdf", "emission"]);
    set(&mut graph, 1, "fac", Value::Float(0.3));
    link(&mut graph, (2, "BSDF"), (1, "Closure1"));
    link(&mut graph, (3, "Emission"), (1, "Closure2"));
    link(&mut graph, (1, "Closure"), (0, "Surface"));
    transform(&mut graph, false);
    assert_eq!(kind_list(&graph), ["output", "mix_closure", "diffuse_bsdf", "emission", "mix_closure_weight"]);
    assert_eq!((float(&graph, 4, "fac"), source(&graph, 4, "Weight")), (0.3, None));
    assert_eq!((source(&graph, 2, "SurfaceMixWeight"), source(&graph, 3, "SurfaceMixWeight")), (Some((4, "Weight1")), Some((4, "Weight2"))));
}

#[test]
fn a_factor_that_is_linked_feeds_the_weights() {
    let mut graph = graph_of(&["mix_closure", "diffuse_bsdf", "noise_texture"]);
    link(&mut graph, (3, "Fac"), (1, "Fac"));
    link(&mut graph, (2, "BSDF"), (1, "Closure1"));
    link(&mut graph, (1, "Closure"), (0, "Surface"));
    transform(&mut graph, false);
    assert_eq!((source(&graph, 4, "Fac"), source(&graph, 2, "SurfaceMixWeight")), (Some((3, "Fac")), Some((4, "Weight1"))));
}

#[test]
fn an_add_passes_the_weight_on_to_both_sides() {
    let mut graph = graph_of(&["add_closure", "diffuse_bsdf", "emission"]);
    link(&mut graph, (2, "BSDF"), (1, "Closure1"));
    link(&mut graph, (3, "Emission"), (1, "Closure2"));
    link(&mut graph, (1, "Closure"), (0, "Surface"));
    transform(&mut graph, false);
    assert_eq!((float(&graph, 2, "surface_mix_weight"), float(&graph, 3, "surface_mix_weight"), kind_list(&graph).len()), (1.0, 1.0, 4));
}

#[test]
fn a_mix_under_a_mix_takes_its_share_of_the_weight_above() {
    let mut graph = graph_of(&["mix_closure", "mix_closure", "diffuse_bsdf", "emission"]);
    link(&mut graph, (2, "Closure"), (1, "Closure1"));
    link(&mut graph, (3, "BSDF"), (2, "Closure1"));
    link(&mut graph, (4, "Emission"), (1, "Closure2"));
    link(&mut graph, (1, "Closure"), (0, "Surface"));
    transform(&mut graph, false);
    // The outer mix's weights, then the inner's, fed by the outer's first.
    assert_eq!(of_kind(&graph, "mix_closure_weight"), [5, 6]);
    assert_eq!((source(&graph, 6, "Weight"), source(&graph, 3, "SurfaceMixWeight"), source(&graph, 4, "SurfaceMixWeight")), (Some((5, "Weight1")), Some((6, "Weight1")), Some((5, "Weight2"))));
}

#[test]
fn a_closure_reached_twice_has_its_weights_added() {
    let mut graph = graph_of(&["add_closure", "diffuse_bsdf"]);
    link(&mut graph, (2, "BSDF"), (1, "Closure1"));
    link(&mut graph, (2, "BSDF"), (1, "Closure2"));
    link(&mut graph, (1, "Closure"), (0, "Surface"));
    transform(&mut graph, false);
    // First the whole weight as a value, then a sum of it and one more.
    assert_eq!(kind_list(&graph), ["output", "add_closure", "diffuse_bsdf", "math"]);
    assert_eq!((float(&graph, 3, "value1"), float(&graph, 3, "value2"), source(&graph, 2, "SurfaceMixWeight")), (1.0, 1.0, Some((3, "Value"))));
}

#[test]
fn a_volume_s_closures_have_a_weight_of_their_own() {
    let mut graph = graph_of(&["emission"]);
    link(&mut graph, (1, "Emission"), (0, "Volume"));
    transform(&mut graph, true);
    assert_eq!((float(&graph, 1, "volume_mix_weight"), float(&graph, 1, "surface_mix_weight")), (1.0, 0.0));
}

#[test]
fn a_node_that_is_no_closure_is_left_alone() {
    let mut graph = graph_of(&["math"]);
    let before = graph.clone();
    graph.transform_multi_closure(1, None, false);
    assert_eq!(graph, before);
}
