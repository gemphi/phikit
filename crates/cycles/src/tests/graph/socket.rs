// SPDX-License-Identifier: Apache-2.0
use crate::graph::*;

#[test]
fn a_kind_is_named_as_its_socket_macro() {
    assert_eq!(SocketKind::named("boolean"), SocketKind::Boolean);
    assert_eq!(SocketKind::named("float_array"), SocketKind::FloatArray);
    assert_eq!(SocketKind::named("no_such_kind"), SocketKind::Undefined);
}

#[test]
fn every_socket_of_cycles_has_a_kind_but_the_one_quaternion() {
    let undefined: Vec<String> = NodeType::all().iter().flat_map(|node| node.socket.iter().filter(|socket| socket.kind == SocketKind::Undefined).map(|socket| format!("{}.{}", node.name, socket.name))).collect();
    // `SOCKET_OUT_QUATERNION` has no kind of its own in `SocketType::Type`.
    assert_eq!(undefined, ["particle_info.rotation"]);
}

#[test]
fn a_shader_node_s_sockets_are_inputs_and_outputs() {
    let emission = NodeType::find("emission").expect("in data/node.tsv");
    let direction = |name: &str| emission.socket.iter().find(|socket| socket.name == name).map(|socket| socket.direction);
    assert_eq!(direction("color"), Some(SocketDirection::Input));
    assert_eq!(direction("emission"), Some(SocketDirection::Output));
    let integrator = NodeType::find("integrator").expect("in data/node.tsv");
    assert!(integrator.socket.iter().all(|socket| socket.direction == SocketDirection::Setting));
}

#[test]
fn a_socket_keeps_its_label_and_its_default_as_cycles_writes_it() {
    let integrator = NodeType::find("integrator").expect("in data/node.tsv");
    let socket = integrator.socket.iter().find(|socket| socket.name == "ao_distance").expect("the integrator's");
    assert_eq!((socket.label.as_str(), socket.default_text.as_str(), socket.kind), ("AO Distance", "FLT_MAX", SocketKind::Float));
    assert_eq!(socket.default, Value::Float(f32::MAX));
}
