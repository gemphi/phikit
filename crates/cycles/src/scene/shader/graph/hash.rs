// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/shader_graph.cpp`
//! (`ShaderGraph::compute_displacement_hash`): a digest of every node the
//! displacement depends on and of how they are linked, by which a mesh
//! is told that it must be displaced again.
//!
//! Not ported: the code of an OSL node, which Cycles takes in too.

use std::collections::BTreeSet;

use super::ShaderGraph;
use crate::util::Md5Hash;

impl ShaderGraph {
    /// `compute_displacement_hash`.
    pub fn compute_displacement_hash(&mut self) {
        let displacement = self.input_of(self.output(), "Displacement").filter(|input| self.link_of(*input).is_some());
        let Some(displacement) = displacement else {
            self.displacement_hash = String::new();
            return;
        };
        let mut displace = BTreeSet::new();
        self.find_dependency(&mut displace, displacement);
        let mut md5 = Md5Hash::new();
        for node in displace.into_iter().filter_map(|id| self.node(id)) {
            node.node.hash(&mut md5);
            // Each input: the node it is linked to and that node's output.
            for from in node.input.iter().map(|input| input.link) {
                let id = from.map(|from| from.node as i32).unwrap_or(0);
                md5.append(&id.to_le_bytes());
                md5.append_str(from.and_then(|from| self.output_socket(from)).map(|output| output.name()).unwrap_or(""));
            }
        }
        self.displacement_hash = md5.hex();
    }
}

#[cfg(test)]
mod test;
