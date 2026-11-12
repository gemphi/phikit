// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/shader_graph.cpp`
//! (`ShaderGraph::deduplicate_nodes`, `optimize_volume_output`,
//! `break_cycles`, `clean`): the graph made smaller without changing what
//! it computes. Nodes that are the same become one, a volume output with
//! nothing a volume can be made of is unlinked, cycles are broken, and
//! the nodes the output does not depend on are dropped.
//!
//! Not ported: `constant_fold` and `simplify_settings`, the first two
//! steps of `clean`, which are the node classes' own.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::{InputId, OutputId, ShaderGraph, ShaderNode, ShaderNodeSpecialType};

impl ShaderGraph {
    /// `check_node_inputs_traversed`: every node this one takes a value
    /// from is done.
    fn input_traversed(&self, node: usize, done: &BTreeSet<usize>) -> bool {
        let link = self.node(node).into_iter().flat_map(|node| node.input.iter().filter_map(|input| input.link));
        link.into_iter().all(|from| done.contains(&from.node))
    }

    /// `deduplicate_nodes`: a node equal to one met before, in its
    /// settings and in what its inputs are linked to, gives way to it.
    /// The nodes are met from those that depend on nothing upwards, so
    /// what a node depends on is deduplicated before it is looked at.
    pub fn deduplicate_node(&mut self) {
        let free = |node: &&ShaderNode| node.input.iter().all(|input| input.link.is_none());
        let mut queue: VecDeque<usize> = self.node_list().filter(free).map(|node| node.id).collect();
        let mut scheduled: BTreeSet<usize> = queue.iter().copied().collect();
        let mut done = BTreeSet::new();
        let mut candidate: BTreeMap<&'static str, BTreeSet<usize>> = BTreeMap::new();
        while let Some(id) = queue.pop_front() {
            done.insert(id);
            let Some(node) = self.node(id) else {
                continue;
            };
            let linked: Vec<InputId> = node.output.iter().flat_map(|output| output.link.iter().copied()).collect();
            for input in &linked {
                if !scheduled.contains(&input.node) && self.input_traversed(input.node, &done) {
                    queue.push_back(input.node);
                    scheduled.insert(input.node);
                }
            }
            // Only a node something uses is worth merging.
            if linked.is_empty() {
                continue;
            }
            let output_count = node.output.len();
            let known = candidate.entry(node.type_name()).or_default();
            let twin = known.iter().copied().find(|other| *other != id && self.node(*other).is_some_and(|other| node.equals(other)));
            match twin {
                Some(twin) => (0..output_count).for_each(|socket| self.relink_node(id, OutputId { node: id, socket }, Some(OutputId { node: twin, socket }))),
                None => {
                    known.insert(id);
                }
            }
        }
    }

    /// `optimize_volume_output`: unlink the volume when nothing under it
    /// can make a volume, and tell each attribute node whose value goes
    /// through something nonlinear that it cannot be sampled at random.
    pub fn optimize_volume_output(&mut self) {
        let Some(from) = self.input_of(self.output(), "Volume").and_then(|input| self.link_of(input)) else {
            return;
        };
        let mut has_valid_volume = false;
        let mut scheduled = BTreeSet::from([(from.node, false)]);
        let mut queue = VecDeque::from([(from.node, false)]);
        while let Some((id, nonlinear)) = queue.pop_front() {
            let linear = self.node(id).is_some_and(ShaderNode::is_linear_operation);
            let Some(node) = self.node_mut(id) else {
                continue;
            };
            if nonlinear && node.type_name() == "attribute" {
                node.stochastic_sample = false;
            }
            let nonlinear = nonlinear || !linear;
            has_valid_volume |= node.has_volume_support();
            for below in node.input.iter().filter_map(|input| input.link).map(|from| (from.node, nonlinear)) {
                if scheduled.insert(below) {
                    queue.push_back(below);
                }
            }
        }
        if !has_valid_volume {
            self.disconnect_output(from);
        }
    }

    /// `break_cycles`: walk down from `node`, and unlink an input that
    /// leads back to a node the walk is still under. Which link of a
    /// cycle goes is not defined; a cycle is not a valid graph.
    pub fn break_cycle(&mut self, node: usize, visited: &mut [bool], on_stack: &mut [bool]) {
        (visited[node], on_stack[node]) = (true, true);
        let count = self.node(node).map(|node| node.input.len()).unwrap_or(0);
        for input in (0..count).map(|socket| InputId { node, socket }) {
            match self.link_of(input).map(|from| (from.node, on_stack[from.node], visited[from.node])) {
                Some((_, true, _)) => self.disconnect_input(input),
                Some((below, false, false)) => self.break_cycle(below, visited, on_stack),
                _ => {}
            }
        }
        on_stack[node] = false;
    }

    /// `clean`: deduplicate, look at the volume, break cycles, and drop
    /// every node neither the output nor an AOV output depends on.
    pub fn clean(&mut self) {
        self.deduplicate_node();
        self.optimize_volume_output();
        let mut visited = vec![false; self.node_id_count()];
        let mut on_stack = vec![false; self.node_id_count()];
        self.break_cycle(self.output(), &mut visited, &mut on_stack);
        let aov: Vec<usize> = self.node_list().filter(|node| node.special_type == ShaderNodeSpecialType::OutputAov).map(|node| node.id).collect();
        aov.into_iter().for_each(|node| self.break_cycle(node, &mut visited, &mut on_stack));

        let unused: Vec<usize> = self.node_list().map(|node| node.id).filter(|id| !visited[*id]).collect();
        for node in &unused {
            let count = self.node(*node).map(|node| node.input.len()).unwrap_or(0);
            (0..count).for_each(|socket| self.disconnect_input(InputId { node: *node, socket }));
        }
        self.remove_node(unused);
    }
}

#[cfg(test)]
mod test;
