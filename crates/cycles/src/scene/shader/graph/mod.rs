// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/shader_graph.h` and `shader_graph.cpp`
//! (`ShaderGraph`): the nodes of a shader and the links between them, and
//! what is done to the graph before it is compiled. The first node is the
//! output, with the surface, the volume and the displacement as inputs.
//!
//! ```text
//!  link.rs      connect, disconnect, relink; a convert node where types differ
//!  proxy.rs     remove_proxy_nodes: group sockets passed through
//!  input.rs     default_inputs: texture coordinates and normals nobody linked
//!  clean.rs     deduplicate, drop a volume of nothing, break cycles, drop unused nodes
//!  bump.rs      three copies of what feeds a bump node; bump from displacement
//!  closure.rs   transform_multi_closure: mixes and adds become weights
//!  hash.rs      compute_displacement_hash
//! ```
//!
//! Cycles keeps the nodes in a list and links them by pointer. Here a
//! node keeps the place its id names, which is left empty when the node
//! is removed, so the nodes are still met in the order they were made.
//! Not ported: `expand`, constant folding and `simplify_settings`, which
//! are the node classes' own; `get_num_closures`, which needs their
//! closure types; `dump_graph`.

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

pub use node::{ShaderNode, ShaderNodeSpecialType};
pub use socket::{InputId, OutputId, ShaderBump, ShaderInput, ShaderOutput};

use std::collections::{BTreeMap, BTreeSet};

/// `ShaderGraph`.
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderGraph {
    /// `nodes`, each at the place its id names.
    node: Vec<Option<ShaderNode>>,
    /// Nothing more may be changed: it is ready to compile.
    pub finalized: bool,
    pub simplified: bool,
    /// What the displacement depends on, as a digest; empty when nothing
    /// is linked to the displacement.
    pub displacement_hash: String,
}

impl Default for ShaderGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderGraph {
    /// `ShaderGraph()`: a graph of an output node alone.
    pub fn new() -> ShaderGraph {
        let mut graph = ShaderGraph { node: Vec::new(), finalized: false, simplified: false, displacement_hash: String::new() };
        let made = graph.create_node("output");
        debug_assert_eq!(made, Some(0), "data/node.tsv has the output node");
        graph
    }

    /// `add_node`: take a node into the graph and give it its id.
    pub fn add_node(&mut self, mut node: ShaderNode) -> usize {
        debug_assert!(!self.finalized);
        self.simplified = false;
        node.id = self.node.len();
        self.node.push(Some(node));
        self.node.len() - 1
    }

    /// `create_node`: a new node of the type named so; its id.
    pub fn create_node(&mut self, type_name: &str) -> Option<usize> {
        ShaderNode::new(type_name).map(|node| self.add_node(node))
    }

    /// `output`: the output node's id.
    pub fn output(&self) -> usize {
        0
    }

    /// `num_node_ids`: how many nodes were ever added.
    pub fn node_id_count(&self) -> usize {
        self.node.len()
    }

    /// The node with this id, unless it was removed.
    pub fn node(&self, id: usize) -> Option<&ShaderNode> {
        self.node.get(id)?.as_ref()
    }

    pub fn node_mut(&mut self, id: usize) -> Option<&mut ShaderNode> {
        self.node.get_mut(id)?.as_mut()
    }

    /// `nodes`: the nodes in the graph, in the order they were made.
    pub fn node_list(&self) -> impl Iterator<Item = &ShaderNode> {
        self.node.iter().flatten()
    }

    /// Take these nodes out of the graph. Their links must be gone.
    fn remove_node(&mut self, id: impl IntoIterator<Item = usize>) {
        id.into_iter().for_each(|id| self.node.get_mut(id).into_iter().for_each(|place| *place = None));
    }

    /// `node->input(name)`.
    pub fn input_of(&self, node: usize, name: &str) -> Option<InputId> {
        Some(InputId { node, socket: self.node(node)?.input_named(name)? })
    }

    /// `node->output(name)`.
    pub fn output_of(&self, node: usize, name: &str) -> Option<OutputId> {
        Some(OutputId { node, socket: self.node(node)?.output_named(name)? })
    }

    pub fn input(&self, id: InputId) -> Option<&ShaderInput> {
        self.node(id.node)?.input.get(id.socket)
    }

    pub fn output_socket(&self, id: OutputId) -> Option<&ShaderOutput> {
        self.node(id.node)?.output.get(id.socket)
    }

    /// `input->link`: the output an input takes its value from.
    pub fn link_of(&self, id: InputId) -> Option<OutputId> {
        self.input(id)?.link
    }

    /// `output->links`: the inputs an output feeds, as they are now.
    pub fn linked(&self, id: OutputId) -> Vec<InputId> {
        self.output_socket(id).map(|output| output.link.clone()).unwrap_or_default()
    }

    /// `find_dependencies`: every node this input depends on, directly or
    /// through others, added to `found`.
    pub fn find_dependency(&self, found: &mut BTreeSet<usize>, input: InputId) {
        let Some(from) = self.link_of(input).filter(|from| !found.contains(&from.node)) else {
            return;
        };
        let count = self.node(from.node).map(|node| node.input.len()).unwrap_or(0);
        (0..count).for_each(|socket| self.find_dependency(found, InputId { node: from.node, socket }));
        found.insert(from.node);
    }

    /// `copy_nodes`: a copy of each of these nodes, linked to each other
    /// as the originals are. Every node one of them takes a value from
    /// must be among them. The answer is each node's copy.
    pub fn copy_node(&mut self, original: &BTreeSet<usize>) -> BTreeMap<usize, usize> {
        let mut copied = BTreeMap::new();
        for id in original {
            let copy = self.node(*id).map(ShaderNode::copy);
            copy.into_iter().for_each(|copy| {
                copied.insert(*id, self.add_node(copy));
            });
        }
        for (id, copy) in &copied {
            let link: Vec<Option<OutputId>> = self.node(*id).map(|node| node.input.iter().map(|input| input.link).collect()).unwrap_or_default();
            for (socket, from) in link.into_iter().enumerate() {
                let from = from.and_then(|from| Some(OutputId { node: *copied.get(&from.node)?, socket: from.socket }));
                from.into_iter().for_each(|from| self.connect(from, InputId { node: *copy, socket }));
            }
        }
        copied
    }

    /// `simplify`: default inputs, the clean-up, and the bump nodes'
    /// samples. `use_osl` is whether the shaders are run by OSL.
    pub fn simplify(&mut self, use_osl: bool) {
        if !self.simplified {
            self.default_input(use_osl);
            self.clean();
            self.refine_bump_node();
            self.simplified = true;
        }
    }

    /// `finalize`: make the graph ready to compile, once. With `do_bump`
    /// the displacement also bends the normal. The mixes and adds of
    /// closures under the surface and the volume become weights.
    pub fn finalize(&mut self, use_osl: bool, do_bump: bool, bump_in_object_space: bool) {
        if self.finalized {
            return;
        }
        self.simplify(use_osl);
        if do_bump {
            self.bump_from_displacement(bump_in_object_space);
        }
        for (name, volume) in [("Surface", false), ("Volume", true)] {
            let from = self.input_of(self.output(), name).and_then(|input| self.link_of(input));
            from.into_iter().for_each(|from| self.transform_multi_closure(from.node, None, volume));
        }
        self.finalized = true;
    }
}
