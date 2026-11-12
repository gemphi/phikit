// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/shader_graph.cpp`
//! (`ShaderInput::disconnect`, `ShaderOutput::disconnect`,
//! `ShaderGraph::connect`, `disconnect`, `relink`): the links of a graph.
//! An output of one type linked to an input of another gets a node
//! between them that converts.
//!
//! Not ported: the warnings Cycles logs where a link is refused.

use super::{InputId, OutputId, ShaderGraph, ShaderInput, ShaderNodeSpecialType, ShaderOutput};
use crate::graph::{NodeType, SocketDirection, SocketKind, Value};

/// The convert node type from one socket type to another, of those
/// `ConvertNode::get_node_types` makes.
fn convert_type(from: SocketKind, to: SocketKind) -> Option<&'static str> {
    let kind = |node: &NodeType, direction: SocketDirection| node.socket.iter().find(|socket| socket.direction == direction).map(|socket| socket.kind);
    let fits = |node: &&NodeType| node.class == "ConvertNode" && kind(node, SocketDirection::Input) == Some(from) && kind(node, SocketDirection::Output) == Some(to);
    NodeType::all().iter().find(fits).map(|node| node.name.as_str())
}

impl ShaderGraph {
    fn input_mut(&mut self, id: InputId) -> Option<&mut ShaderInput> {
        self.node_mut(id.node)?.input.get_mut(id.socket)
    }

    fn output_mut(&mut self, id: OutputId) -> Option<&mut ShaderOutput> {
        self.node_mut(id.node)?.output.get_mut(id.socket)
    }

    /// `connect`: the input takes its value from the output. An input
    /// that is linked already stays as it is. Where the types differ a
    /// convert node goes between; a colour or a number into a closure
    /// goes through an emission node; a closure converts to nothing, so
    /// that link is not made.
    pub fn connect(&mut self, from: OutputId, to: InputId) {
        debug_assert!(!self.finalized);
        let (Some(from_kind), Some(to_kind)) = (self.output_socket(from).map(ShaderOutput::kind), self.input(to).map(ShaderInput::kind)) else {
            return;
        };
        if self.link_of(to).is_some() {
            return;
        }
        let between = match (from_kind == to_kind, from_kind == SocketKind::Closure, to_kind == SocketKind::Closure) {
            (true, ..) => {
                self.input_mut(to).into_iter().for_each(|input| input.link = Some(from));
                self.output_mut(from).into_iter().for_each(|output| output.link.push(to));
                None
            }
            (false, true, _) => None,
            (false, false, true) => self.create_node("emission").map(|emission| {
                self.node_mut(emission).into_iter().for_each(|node| {
                    node.from_auto_conversion = true;
                    node.node.set("color", Value::Float3([1.0; 3]));
                    node.node.set("strength", Value::Float(1.0));
                });
                // A number goes to the strength, which saves turning it
                // into a colour first.
                let name = match from_kind == SocketKind::Float {
                    true => "Strength",
                    false => "Color",
                };
                (emission, self.input_of(emission, name))
            }),
            (false, false, false) => convert_type(from_kind, to_kind).and_then(|name| self.create_node(name)).map(|convert| {
                self.node_mut(convert).into_iter().for_each(|node| node.special_type = ShaderNodeSpecialType::Autoconvert);
                (convert, Some(InputId { node: convert, socket: 0 }))
            }),
        };
        match between {
            Some((node, Some(input))) => {
                self.connect(from, input);
                self.connect(OutputId { node, socket: 0 }, to);
            }
            _ => {}
        }
    }

    /// `disconnect` of an input: it takes its value from no output.
    pub fn disconnect_input(&mut self, to: InputId) {
        debug_assert!(!self.finalized);
        self.simplified = false;
        let from = self.input_mut(to).and_then(|input| input.link.take());
        from.and_then(|from| self.output_mut(from)).into_iter().for_each(|output| output.link.retain(|input| *input != to));
    }

    /// `disconnect` of an output: no input takes its value from it.
    pub fn disconnect_output(&mut self, from: OutputId) {
        debug_assert!(!self.finalized);
        self.simplified = false;
        let linked = self.output_mut(from).map(|output| std::mem::take(&mut output.link)).unwrap_or_default();
        for to in linked {
            self.input_mut(to).into_iter().for_each(|input| input.link = None);
        }
    }

    /// `relink` of two inputs: `to` takes over what `from` is linked to,
    /// and its value.
    pub fn relink_input(&mut self, from: InputId, to: InputId) {
        match self.link_of(from) {
            Some(out) => {
                self.disconnect_input(from);
                self.connect(out, to);
            }
            None => {}
        }
        let name = (self.input(from).map(ShaderInput::socket_name), self.input(to).map(ShaderInput::socket_name));
        let source = self.node(from.node).map(|node| node.node.clone());
        match (name, source, self.node_mut(to.node)) {
            ((Some(from_name), Some(to_name)), Some(source), Some(node)) => node.node.copy_value(to_name, &source, from_name),
            _ => {}
        }
    }

    /// `relink` of two outputs: what took its value from `from` takes it
    /// from `to`, or from nothing.
    pub fn relink_output(&mut self, from: OutputId, to: Option<OutputId>) {
        for input in self.linked(from) {
            self.disconnect_input(input);
            to.into_iter().for_each(|to| self.connect(to, input));
        }
    }

    /// `relink` around a node: the node is bypassed. Its inputs are
    /// unlinked, and what took its value from its output `from` takes it
    /// from `to`.
    pub fn relink_node(&mut self, node: usize, from: OutputId, to: Option<OutputId>) {
        let linked = self.linked(from);
        let count = self.node(node).map(|node| node.input.len()).unwrap_or(0);
        for socket in 0..count {
            self.disconnect_input(InputId { node, socket });
        }
        for input in linked {
            self.disconnect_input(input);
            to.into_iter().for_each(|to| self.connect(to, input));
        }
        self.simplified = false;
    }
}

#[cfg(test)]
mod test;
