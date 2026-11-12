// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/blender/shader.cpp`: `convert_socket_type`
//! (201), `set_default_value` (223), `node_find_input_by_name` (1212),
//! `node_find_output_by_name` (1283), `add_nodes_inlined` (1334) and
//! `add_nodes` (1525): a material's node tree into a [`ShaderGraph`]. A
//! muted node or a reroute becomes a proxy for each of its internal
//! links; a node group becomes a proxy for each of its sockets with its
//! own tree added between them (Cycles inlines the groups first,
//! `inline_shader_node_tree`, with the same result).
//!
//! Not ported: the muted node's internal links are Blender's
//! `node_internal_links`; here each output takes the first input of its
//! type. A link's `NODE_LINK_VALID` and muted flags are not in the source.
use std::collections::HashMap;
use super::graph::{GraphSource, NodeSource, SocketClass, SocketSource, SocketValue};
use super::node::add_node;
use crate::graph::{SocketKind, Value};
use crate::scene::{InputId, OutputId, ShaderGraph};

/// `convert_socket_type`, as the name a convert node's type is made with.
fn type_name(class: SocketClass) -> Option<&'static str> {
    match class {
        SocketClass::Float => Some("float"),
        SocketClass::Boolean | SocketClass::Int => Some("int"),
        SocketClass::Vector => Some("vector"),
        SocketClass::Color => Some("color"),
        SocketClass::String => Some("string"),
        SocketClass::Closure => Some("closure"),
        SocketClass::Undefined => None,
    }
}

/// `set_default_value`: the value an unlinked input holds.
fn set_default(graph: &mut ShaderGraph, input: InputId, b_sock: &SocketSource) {
    let Some(socket) = graph.input(input).map(|input| (input.socket_name(), input.kind())) else { return };
    let value = match (socket.1, b_sock.value) {
        (SocketKind::Float, Some(SocketValue::Float(value))) => Value::Float(value),
        (SocketKind::Int, Some(SocketValue::Int(value))) => Value::Int(value),
        (SocketKind::Int, Some(SocketValue::Boolean(value))) => Value::Int(value as i32),
        (SocketKind::Color, Some(SocketValue::Color(value))) => Value::Float3(value),
        (SocketKind::Vector | SocketKind::Normal | SocketKind::Point, Some(SocketValue::Vector(value))) => Value::Float3(value),
        _ => return,
    };
    if let Some(node) = graph.node_mut(input.node) {
        node.node.set(socket.0, value);
    }
}

/// A proxy node: a convert node from a type to itself.
fn proxy(graph: &mut ShaderGraph, class: SocketClass) -> Option<usize> {
    let name = type_name(class)?;
    graph.create_node(&format!("convert_{name}_to_{name}"))
}

/// The renames of `node_find_input_by_name`.
fn input_of(graph: &ShaderGraph, b_node: &NodeSource, node: usize, identifier: &str) -> Option<InputId> {
    let mut name = identifier.to_string();
    if let Some(input) = graph.input_of(node, &name) {
        return Some(input);
    }
    if name.starts_with("Shader") {
        name = name.replacen("Shader", "Closure", 1);
    }
    if b_node.idname == "ShaderNodeMix" {
        for (from, to) in [("Factor_Float", "Factor"), ("Factor_Vector", "Factor"), ("A_Float", "A"), ("B_Float", "B"), ("A_Color", "A"), ("B_Color", "B"), ("A_Vector", "A"), ("B_Vector", "B")] {
            if name.ends_with(from) {
                name = name.replacen(from, to, 1);
                break;
            }
        }
    }
    if let Some(input) = graph.input_of(node, &name) {
        return Some(input);
    }
    let renamed = [("_001", "2"), (".001", "2"), ("_002", "3"), (".002", "3")].into_iter().find(|(from, _)| name.ends_with(from));
    match renamed {
        Some((from, to)) => name = name.replacen(from, to, 1),
        None => name.push('1'),
    }
    graph.input_of(node, &name)
}

/// `node_find_output_by_name`.
fn output_of(graph: &ShaderGraph, b_node: &NodeSource, node: usize, identifier: &str) -> Option<OutputId> {
    if let Some(output) = graph.output_of(node, identifier) {
        return Some(output);
    }
    let mut name = identifier.to_string();
    if name == "Shader" {
        name = "Closure".into();
    }
    if b_node.idname == "ShaderNodeMix" {
        for from in ["Result_Float", "Result_Color", "Result_Vector"] {
            if name.ends_with(from) {
                name = name.replacen(from, "Result", 1);
            }
        }
    }
    graph.output_of(node, &name)
}

type InputMap = HashMap<(String, String), Vec<InputId>>;
type OutputMap = HashMap<(String, String), OutputId>;
type ProxyMap = HashMap<String, usize>;

/// `add_nodes_inlined`: the nodes of `tree` added to `graph`, the
/// group sockets bound to `proxy_input` and `proxy_output`, then its links.
fn add_nodes(graph: &mut ShaderGraph, tree: &GraphSource, is_root: bool, proxy_input: &ProxyMap, proxy_output: &ProxyMap) {
    let mut input_map = InputMap::new();
    let mut output_map = OutputMap::new();
    let key = |node: &NodeSource, socket: &SocketSource| (node.name.clone(), socket.identifier.clone());
    for b_node in &tree.node {
        if b_node.muted || b_node.reroute {
            // Each output takes the first available input of its type.
            for output in b_node.output.iter().filter(|socket| socket.available) {
                let from = b_node.input.iter().filter(|socket| socket.available).find(|socket| socket.class == output.class);
                let (Some(from), Some(id)) = (from, proxy(graph, output.class)) else { continue };
                input_map.entry(key(b_node, from)).or_default().push(InputId { node: id, socket: 0 });
                output_map.insert(key(b_node, output), OutputId { node: id, socket: 0 });
            }
        } else if let Some(group) = b_node.idname.strip_prefix("ShaderNodeGroup").map(|_| &b_node.group) {
            let (mut inner_input, mut inner_output) = (ProxyMap::new(), ProxyMap::new());
            for b_input in &b_node.input {
                let Some(id) = proxy(graph, b_input.class) else { continue };
                inner_input.insert(b_input.identifier.clone(), id);
                input_map.entry(key(b_node, b_input)).or_default().push(InputId { node: id, socket: 0 });
                set_default(graph, InputId { node: id, socket: 0 }, b_input);
            }
            for b_output in &b_node.output {
                let Some(id) = proxy(graph, b_output.class) else { continue };
                inner_output.insert(b_output.identifier.clone(), id);
                output_map.insert(key(b_node, b_output), OutputId { node: id, socket: 0 });
            }
            if let Some(group) = group {
                add_nodes(graph, group, false, &inner_input, &inner_output);
            }
        } else if b_node.idname == "NodeGroupInput" {
            for b_output in &b_node.output {
                if let Some(id) = proxy_input.get(&b_output.identifier) {
                    output_map.insert(key(b_node, b_output), OutputId { node: *id, socket: 0 });
                }
            }
        } else if b_node.idname == "NodeGroupOutput" {
            if b_node.active_output {
                for b_input in &b_node.input {
                    if let Some(id) = proxy_output.get(&b_input.identifier) {
                        input_map.entry(key(b_node, b_input)).or_default().push(InputId { node: *id, socket: 0 });
                        set_default(graph, InputId { node: *id, socket: 0 }, b_input);
                    }
                }
            }
        } else {
            let is_output = is_root && tree.output.as_deref() == Some(b_node.name.as_str());
            let Some(id) = (if is_output { Some(graph.output()) } else { add_node(graph, b_node) }) else { continue };
            for b_input in b_node.input.iter().filter(|socket| socket.available) {
                if let Some(input) = input_of(graph, b_node, id, &b_input.identifier) {
                    input_map.entry(key(b_node, b_input)).or_default().push(input);
                    set_default(graph, input, b_input);
                }
            }
            for b_output in b_node.output.iter().filter(|socket| socket.available) {
                if let Some(output) = output_of(graph, b_node, id, &b_output.identifier) {
                    output_map.insert(key(b_node, b_output), output);
                }
            }
        }
    }
    for link in &tree.link {
        let Some(output) = output_map.get(&(link.from.clone(), link.from_socket.clone())).copied() else { continue };
        for input in input_map.get(&(link.to.clone(), link.to_socket.clone())).cloned().unwrap_or_default() {
            graph.connect(output, input);
        }
    }
}

/// The shader graph of a material's node tree.
pub fn graph_of(tree: &GraphSource) -> ShaderGraph {
    let mut graph = ShaderGraph::new();
    add_nodes(&mut graph, tree, true, &ProxyMap::new(), &ProxyMap::new());
    graph
}

#[cfg(test)]
mod test;
