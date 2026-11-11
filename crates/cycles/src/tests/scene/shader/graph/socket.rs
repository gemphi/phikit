// SPDX-License-Identifier: Apache-2.0
use super::*;

#[test]
fn an_input_is_known_by_its_label_and_holds_its_value_by_its_name() {
    let graph = ShaderGraph::new();
    let surface = graph.input(graph.input_of(0, "Surface").expect("the surface")).expect("the socket");
    assert_eq!((surface.name(), surface.socket_name(), surface.kind()), ("Surface", "surface", SocketKind::Closure));
    assert!(surface.flag() & flag("SocketType::LINKABLE") != 0 && surface.link.is_none() && !surface.constant_folded_in);
}

#[test]
fn an_input_says_which_default_link_it_asks_for() {
    let graph = graph_of(&["diffuse_bsdf", "noise_texture"]);
    let asks = |node: usize, name: &str| graph.input(graph.input_of(node, name).expect("the input")).expect("the socket").flag();
    assert!(asks(1, "Normal") & flag("SocketType::LINK_NORMAL") != 0);
    assert!(asks(2, "Vector") & flag("SocketType::LINK_TEXTURE_GENERATED") != 0);
    assert_eq!(asks(1, "Color") & flag("SocketType::DEFAULT_LINK_MASK"), 0);
}

#[test]
fn an_output_starts_linked_to_nothing() {
    let graph = graph_of(&["noise_texture"]);
    let fac = graph.output_socket(graph.output_of(1, "Fac").expect("the output")).expect("the socket");
    assert_eq!((fac.name(), fac.kind(), fac.link.len()), ("Fac", SocketKind::Float, 0));
    assert_eq!(ShaderBump::default(), ShaderBump::None);
}

#[test]
fn sockets_are_ordered_by_node_then_place() {
    assert!(InputId { node: 1, socket: 5 } < InputId { node: 2, socket: 0 });
    assert!(OutputId { node: 2, socket: 0 } < OutputId { node: 2, socket: 1 });
}
