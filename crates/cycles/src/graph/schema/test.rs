// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::graph::*;

#[test]
fn every_node_type_of_cycles_is_in_the_table() {
    let shader = NodeType::all().iter().filter(|node| node.kind == NodeKind::Shader).count();
    // 97 kinds of shader node, and the 64 convert nodes between 8 types.
    assert_eq!((NodeType::all().len(), shader), (119 + 64, 97 + 64));
    assert!(NodeType::find("no_such_node").is_none());
}

#[test]
fn a_type_knows_its_class_and_the_file_it_comes_from() {
    let integrator = NodeType::find("integrator").expect("in data/node.tsv");
    assert_eq!((integrator.class.as_str(), integrator.file.as_str(), integrator.kind), ("Integrator", "scene/integrator.cpp", NodeKind::None));
    assert_eq!(NodeType::of_class("Film").map(|node| node.name.as_str()), Some("film"));
    assert!(integrator.base_type().is_none());
}

#[test]
fn a_derived_type_has_its_base_s_sockets_first() {
    let spot = NodeType::find("spotlight").expect("in data/node.tsv");
    let point = spot.base_type().expect("a spot light is a point light");
    assert_eq!(point.name, "pointlight");
    let name: Vec<&str> = spot.socket_list().iter().map(|socket| socket.name.as_str()).collect();
    let own: Vec<&str> = spot.socket.iter().map(|socket| socket.name.as_str()).collect();
    assert_eq!(name.len(), point.socket_list().len() + own.len());
    assert!(name.ends_with(&own), "{name:?}");
    assert!(name.contains(&"strength"), "a light's socket: {name:?}");
}

#[test]
fn an_enum_table_gives_a_choice_its_number() {
    let integrator = NodeType::find("integrator").expect("in data/node.tsv");
    let pattern = integrator.enum_named("sampling_pattern_enum").expect("the integrator's");
    assert_eq!(pattern.number("blue_noise_round"), Some(4));
    assert_eq!(pattern.key(1), Some("tabulated_sobol"));
    assert_eq!(pattern.number("no_such_pattern"), None);
    assert!(integrator.enum_named("no_such_enum").is_none());
}

#[test]
fn every_enum_socket_names_a_table_its_type_has() {
    for node in NodeType::all() {
        let enum_socket = node.socket.iter().filter(|socket| socket.kind == SocketKind::Enum && socket.direction != SocketDirection::Output);
        for socket in enum_socket {
            assert!(node.enum_named(&socket.target).is_some(), "{}.{} chooses from {}", node.name, socket.name, socket.target);
        }
    }
}

#[test]
fn a_socket_has_the_flags_cycles_gives_it() {
    let bit = |name: &str| crate::kernel::flag(name);
    let diffuse = NodeType::find("diffuse_bsdf").expect("in data/node.tsv");
    let of = |name: &str| diffuse.socket.iter().find(|socket| socket.name == name).expect("the socket").flag;
    assert_eq!(of("normal"), bit("SocketType::LINKABLE") | bit("SocketType::LINK_NORMAL"));
    assert_eq!((of("color"), of("BSDF")), (bit("SocketType::LINKABLE"), 0));
    // A setting is not linkable.
    let math = NodeType::find("math").expect("in data/node.tsv");
    assert!(math.socket.iter().filter(|socket| socket.direction == SocketDirection::Setting).all(|socket| socket.flag == 0));
}

#[test]
fn there_is_a_convert_node_type_from_each_of_eight_types_to_each() {
    assert_eq!(NodeType::all().iter().filter(|node| node.class == "ConvertNode").count(), 64);
    let node = NodeType::find("convert_color_to_float").expect("in data/node.tsv");
    let socket: Vec<_> = node.socket.iter().map(|socket| (socket.name.as_str(), socket.kind, socket.direction)).collect();
    assert_eq!(socket, [("value_color", SocketKind::Color, SocketDirection::Input), ("value_float", SocketKind::Float, SocketDirection::Output)]);
    assert!(NodeType::find("convert_closure_to_string").is_some() && NodeType::find("convert_boolean_to_float").is_none());
}
