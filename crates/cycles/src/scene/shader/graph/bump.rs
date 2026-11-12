// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/shader_graph.cpp`
//! (`ShaderGraph::refine_bump_nodes`, `bump_from_displacement`): a bump
//! is worked out from a height at three places, the point shaded and one
//! step along each of two directions across the surface. So everything a
//! height depends on is in the graph three times, each copy told which
//! of the three samples it is.

use std::collections::{BTreeMap, BTreeSet};

use super::{OutputId, ShaderBump, ShaderGraph, ShaderNodeSpecialType};
use crate::graph::Value;
use crate::kernel::constant;

impl ShaderGraph {
    /// Tell these nodes which bump sample they are.
    fn set_bump(&mut self, node: impl IntoIterator<Item = usize>, bump: ShaderBump, filter_width: f32) {
        for id in node {
            self.node_mut(id).into_iter().for_each(|node| (node.bump, node.bump_filter_width) = (bump, filter_width));
        }
    }

    /// Link the output `from`, by its label, to the input `to` of `node`.
    fn connect_named(&mut self, from: Option<OutputId>, node: usize, to: &str) {
        let to = self.input_of(node, to);
        from.zip(to).into_iter().for_each(|(from, to)| self.connect(from, to));
    }

    /// The output that is in a copy what `out` is in the original.
    fn copied(copy: &BTreeMap<usize, usize>, out: OutputId) -> Option<OutputId> {
        Some(OutputId { node: *copy.get(&out.node)?, socket: out.socket })
    }

    /// `refine_bump_nodes`: for each bump node with a height linked, what
    /// the height depends on is copied twice, for the two shifted
    /// samples; the original is the centre. The height's link moves to
    /// the centre sample.
    pub fn refine_bump_node(&mut self) {
        let mut id = 0;
        while id < self.node_id_count() {
            let height = self.input_of(id, "Height").filter(|_| self.node(id).is_some_and(|node| node.special_type == ShaderNodeSpecialType::Bump));
            if let Some((height, out)) = height.and_then(|height| Some((height, self.link_of(height)?))) {
                let filter_width = self.node(id).map(|node| node.node.float("filter_width")).unwrap_or(0.0);
                let mut center = BTreeSet::new();
                self.find_dependency(&mut center, height);
                let (dx, dy) = (self.copy_node(&center), self.copy_node(&center));
                self.set_bump(center, ShaderBump::Center, filter_width);
                self.set_bump(dx.values().copied(), ShaderBump::Dx, filter_width);
                self.set_bump(dy.values().copied(), ShaderBump::Dy, filter_width);
                self.connect_named(Self::copied(&dx, out), id, "SampleX");
                self.connect_named(Self::copied(&dy, out), id, "SampleY");
                self.connect_named(Some(out), id, "SampleCenter");
                self.disconnect_input(height);
            }
            id += 1;
        }
    }

    /// `bump_from_displacement`: bend the shading normal the way the
    /// displacement would move the surface. What the displacement depends
    /// on is copied three times; each copy's displacement along the
    /// normal is a height, the three heights go into a bump node, and its
    /// normal becomes the shader's.
    pub fn bump_from_displacement(&mut self, use_object_space: bool) {
        let Some(displacement) = self.input_of(self.output(), "Displacement") else {
            return;
        };
        let Some(out) = self.link_of(displacement) else {
            return;
        };
        let mut displace = BTreeSet::new();
        self.find_dependency(&mut displace, displacement);
        let Some(bump) = self.create_node("bump") else {
            return;
        };
        self.node_mut(bump).into_iter().for_each(|node| {
            node.node.set("use_object_space", Value::Boolean(use_object_space));
            node.node.set("distance", Value::Float(1.0));
        });
        let filter_width = self.node(bump).map(|node| node.node.float("filter_width")).unwrap_or(0.0);
        let copy = [ShaderBump::Center, ShaderBump::Dx, ShaderBump::Dy].map(|sample| {
            let copy = self.copy_node(&displace);
            self.set_bump(copy.values().copied(), sample, filter_width);
            copy
        });
        // Only a bump from displacement may overwrite the shader's normal.
        let set_normal = self.create_node("set_normal");
        // The displacement, a vector, becomes a height along the normal.
        let dot_product = constant("NODE_VECTOR_MATH_DOT_PRODUCT").unwrap_or(0) as i32;
        let dot = [(); 3].map(|_| {
            let dot = self.create_node("vector_math");
            dot.and_then(|dot| self.node_mut(dot)).into_iter().for_each(|node| {
                node.node.set("math_type", Value::Enum(dot_product));
            });
            dot
        });
        let geom = self.create_node("geometry");
        let normal = geom.and_then(|geom| self.output_of(geom, "Normal"));
        self.connect_named(normal, bump, "Normal");
        dot.iter().flatten().for_each(|dot| self.connect_named(normal, *dot, "Vector2"));
        for (copy, dot) in copy.iter().zip(dot) {
            dot.into_iter().for_each(|dot| self.connect_named(Self::copied(copy, out), dot, "Vector1"));
        }
        for (dot, sample) in dot.iter().flatten().zip(["SampleCenter", "SampleX", "SampleY"]) {
            self.connect_named(self.output_of(*dot, "Value"), bump, sample);
        }
        match set_normal {
            Some(set_normal) => {
                self.connect_named(self.output_of(bump, "Normal"), set_normal, "Direction");
                self.connect_named(self.output_of(set_normal, "Normal"), self.output(), "Normal");
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod test;
