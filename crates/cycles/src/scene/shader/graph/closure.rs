// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/shader_graph.cpp`
//! (`ShaderGraph::transform_multi_closure`): for SVM the mixes and adds
//! of closures are not kept as a tree. Each closure node is given the
//! weight the mixes above it leave it, worked out by nodes put in for
//! that, and the closures are then written straight into a list.

use super::{OutputId, ShaderGraph, ShaderNodeSpecialType};
use crate::graph::Value;

impl ShaderGraph {
    /// `transform_multi_closure`: walk down from `node`, a closure or a
    /// mix or add of two, handing each side of a mix its share of
    /// `weight_out`, the weight that reaches the node; nothing means the
    /// whole. `volume` says which of a closure's two weights is meant.
    pub fn transform_multi_closure(&mut self, node: usize, weight_out: Option<OutputId>, volume: bool) {
        let combine = self.node(node).is_some_and(|node| node.special_type == ShaderNodeSpecialType::CombineClosure);
        match combine {
            true => self.transform_combine(node, weight_out, volume),
            false => self.transform_closure(node, weight_out, volume),
        }
    }

    /// A mix closure splits the weight between its two sides by its
    /// factor, through a node made for that; an add closure passes the
    /// weight on to both.
    fn transform_combine(&mut self, node: usize, weight_out: Option<OutputId>, volume: bool) {
        let mut weight = [weight_out, weight_out];
        if let Some(fac) = self.input_of(node, "Fac") {
            let Some(mix) = self.create_node("mix_closure_weight") else {
                return;
            };
            match (self.link_of(fac), self.input_of(mix, "Fac")) {
                (Some(from), Some(to)) => self.connect(from, to),
                _ => {
                    let value = self.node(node).map(|node| node.node.float("fac")).unwrap_or(0.0);
                    self.node_mut(mix).into_iter().for_each(|mix| {
                        mix.node.set("fac", Value::Float(value));
                    });
                }
            }
            let weight_in = self.input_of(mix, "Weight");
            weight_out.zip(weight_in).into_iter().for_each(|(from, to)| self.connect(from, to));
            weight = [self.output_of(mix, "Weight1"), self.output_of(mix, "Weight2")];
        }
        for (name, weight) in ["Closure1", "Closure2"].into_iter().zip(weight) {
            let below = self.input_of(node, name).and_then(|input| self.link_of(input));
            below.into_iter().for_each(|below| self.transform_multi_closure(below.node, weight, volume));
        }
    }

    /// A closure takes the weight that reaches it. One that has a weight
    /// already, by being under two mixes, gets the two added.
    fn transform_closure(&mut self, node: usize, mut weight_out: Option<OutputId>, volume: bool) {
        let name = match volume {
            true => "VolumeMixWeight",
            false => "SurfaceMixWeight",
        };
        // Not a closure node.
        let Some(weight_in) = self.input_of(node, name) else {
            return;
        };
        let socket = self.input(weight_in).map(|input| input.socket_name()).unwrap_or("");
        let value = self.node(node).map(|node| node.node.float(socket)).unwrap_or(0.0);
        let linked = self.link_of(weight_in);
        if linked.is_some() || value != 0.0 {
            let Some(math) = self.create_node("math") else {
                return;
            };
            match (linked, self.input_of(math, "Value1")) {
                (Some(from), Some(to)) => self.connect(from, to),
                _ => self.set_float(math, "value1", value),
            }
            match (weight_out, self.input_of(math, "Value2")) {
                (Some(from), Some(to)) => self.connect(from, to),
                _ => self.set_float(math, "value2", 1.0),
            }
            weight_out = self.output_of(math, "Value");
            self.disconnect_input(weight_in);
        }
        match weight_out {
            Some(from) => self.connect(from, weight_in),
            None => self.set_float(node, socket, value + 1.0),
        }
    }

    /// Give a node's float socket a value.
    fn set_float(&mut self, node: usize, socket: &str, value: f32) {
        self.node_mut(node).into_iter().for_each(|node| {
            node.node.set(socket, Value::Float(value));
        });
    }
}

#[cfg(test)]
mod test;
