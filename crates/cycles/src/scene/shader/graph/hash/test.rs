// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::shader::graph::test::*;

/// A noise texture into a displacement node into the output; the digest.
fn displaced(change: impl FnOnce(&mut ShaderGraph)) -> String {
    let mut graph = graph_of(&["noise_texture", "displacement", "math"]);
    link(&mut graph, (1, "Fac"), (2, "Height"));
    link(&mut graph, (2, "Displacement"), (0, "Displacement"));
    change(&mut graph);
    graph.compute_displacement_hash();
    graph.displacement_hash
}

#[test]
fn nothing_linked_to_the_displacement_is_no_digest() {
    let mut graph = graph_of(&["noise_texture"]);
    graph.displacement_hash = "old".to_string();
    graph.compute_displacement_hash();
    assert!(graph.displacement_hash.is_empty());
}

#[test]
fn the_digest_is_the_same_for_the_same_graph() {
    let digest = displaced(|_| {});
    assert_eq!(digest.len(), 32);
    assert!(digest.chars().all(|digit| digit.is_ascii_digit() || ('A'..='F').contains(&digit)));
    assert_eq!(digest, displaced(|_| {}));
}

#[test]
fn a_changed_value_or_link_under_the_displacement_changes_it() {
    let plain = displaced(|_| {});
    assert_ne!(plain, displaced(|graph| set(graph, 1, "scale", Value::Float(9.0))));
    assert_ne!(plain, displaced(|graph| set(graph, 2, "midlevel", Value::Float(0.25))));
    assert_ne!(plain, displaced(|graph| link(graph, (3, "Value"), (1, "Detail"))));
    // Which output a link comes from matters too.
    let other = displaced(|graph| {
        graph.disconnect_input(graph.input_of(2, "Height").expect("the input"));
        link(graph, (1, "Color"), (2, "Normal"));
    });
    assert_ne!(plain, other);
}

#[test]
fn what_the_displacement_does_not_depend_on_leaves_it_as_it_was() {
    let plain = displaced(|_| {});
    assert_eq!(plain, displaced(|graph| set(graph, 3, "value1", Value::Float(9.0))));
    assert_eq!(plain, displaced(|graph| {
        let diffuse = graph.create_node("diffuse_bsdf").expect("made");
        link(graph, (diffuse, "BSDF"), (0, "Surface"));
    }));
}
