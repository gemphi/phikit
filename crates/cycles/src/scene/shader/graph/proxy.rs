// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/shader_graph.cpp`
//! (`ShaderGraph::remove_proxy_nodes`): a proxy node only passes a value
//! on, as the sockets of a node group do while a scene is read. They are
//! taken out early, so that nothing later sees them.

use super::{InputId, OutputId, ShaderGraph, ShaderInput, ShaderNodeSpecialType};
use crate::kernel::flag;

impl ShaderGraph {
    /// `remove_proxy_nodes`: what a proxy fed takes its value from what
    /// fed the proxy. A proxy fed by nothing hands its own value to what
    /// it fed; a convert node it fed that led only to inputs with a
    /// default link of their own, such as a normal, goes with it.
    pub fn remove_proxy_node(&mut self) {
        let default_link = flag("SocketType::DEFAULT_LINK_MASK");
        let proxy: Vec<usize> = self.node_list().filter(|node| node.special_type == ShaderNodeSpecialType::Proxy).map(|node| node.id).collect();
        let mut removed = proxy.clone();
        for id in proxy {
            let (input, output) = (InputId { node: id, socket: 0 }, OutputId { node: id, socket: 0 });
            if let Some(from) = self.link_of(input) {
                self.relink_node(id, output, Some(from));
                continue;
            }
            let source = self.node(id).map(|node| (node.node.clone(), node.input.first().map(ShaderInput::socket_name)));
            for to in self.linked(output) {
                let auto = self.node(to.node).is_some_and(|node| node.special_type == ShaderNodeSpecialType::Autoconvert);
                if auto {
                    let mut all_removed = true;
                    for after in self.linked(OutputId { node: to.node, socket: 0 }) {
                        match self.input(after).is_some_and(|input| input.flag() & default_link != 0) {
                            true => self.disconnect_input(after),
                            false => all_removed = false,
                        }
                    }
                    if all_removed {
                        removed.push(to.node);
                    }
                }
                self.disconnect_input(to);
                // The value the proxy held goes to the socket it fed.
                let name = self.input(to).map(ShaderInput::socket_name);
                match (&source, name, self.node_mut(to.node)) {
                    (Some((source, Some(source_name))), Some(name), Some(node)) => node.node.copy_value(name, source, source_name),
                    _ => {}
                }
            }
        }
        self.remove_node(removed);
    }
}

#[cfg(test)]
mod test;
