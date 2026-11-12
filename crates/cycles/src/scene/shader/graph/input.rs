// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/shader_graph.cpp`
//! (`ShaderGraph::default_inputs`): an input nobody linked that asks for
//! a default link, such as a texture's coordinate or a closure's normal,
//! is linked to a texture coordinate node or a geometry node. One of
//! each serves the whole graph.

use super::{InputId, OutputId, ShaderGraph};
use crate::graph::Value;
use crate::kernel::{constant, flag};

/// The nodes made so far to link default inputs to.
#[derive(Default)]
struct Source {
    texco: Option<usize>,
    geom: Option<usize>,
    normal_transform: Option<usize>,
}

impl ShaderGraph {
    /// The output `name` of the node at `place`, which is made when it is
    /// first asked for.
    fn source(&mut self, place: &mut Option<usize>, type_name: &str, name: &str) -> Option<OutputId> {
        *place = place.or_else(|| self.create_node(type_name));
        self.output_of((*place)?, name)
    }

    /// The incoming direction as a texture wants it: the geometry's, from
    /// the world into the object's space.
    fn incoming(&mut self, made: &mut Source) -> Option<OutputId> {
        if made.normal_transform.is_none() {
            let from = self.source(&mut made.geom, "geometry", "Incoming");
            let transform = self.create_node("vector_transform")?;
            let setting = [("transform_type", "NODE_VECTOR_TRANSFORM_TYPE_NORMAL"), ("convert_from", "NODE_VECTOR_TRANSFORM_CONVERT_SPACE_WORLD"), ("convert_to", "NODE_VECTOR_TRANSFORM_CONVERT_SPACE_OBJECT")];
            for (socket, name) in setting {
                let number = constant(name).unwrap_or(0) as i32;
                self.node_mut(transform).into_iter().for_each(|node| {
                    node.node.set(socket, Value::Enum(number));
                });
            }
            let to = self.input_of(transform, "Vector");
            from.zip(to).into_iter().for_each(|(from, to)| self.connect(from, to));
            made.normal_transform = Some(transform);
        }
        self.output_of(made.normal_transform?, "Vector")
    }

    /// `default_inputs`. With `do_osl` the inputs only OSL has are linked
    /// too. Nodes made here are looked at in their turn.
    pub fn default_input(&mut self, do_osl: bool) {
        let bit = |name: &str| flag(&format!("SocketType::{name}"));
        let (osl_internal, generated) = (bit("OSL_INTERNAL"), bit("LINK_TEXTURE_GENERATED"));
        // The first of these an input asks for is the one it gets.
        let chain = ["LINK_TEXTURE_NORMAL", "LINK_TEXTURE_UV", "LINK_TEXTURE_INCOMING", "LINK_INCOMING", "LINK_NORMAL", "LINK_POSITION", "LINK_TANGENT"].map(|name| (name, bit(name)));
        let mut made = Source::default();
        let mut id = 0;
        while id < self.node_id_count() {
            let count = self.node(id).map(|node| node.input.len()).unwrap_or(0);
            for input in (0..count).map(|socket| InputId { node: id, socket }) {
                let Some(wanted) = self.input(input).filter(|socket| socket.link.is_none() && (socket.flag() & osl_internal == 0 || do_osl)).map(|socket| socket.flag()) else {
                    continue;
                };
                if wanted & generated != 0 {
                    let from = self.source(&mut made.texco, "texture_coordinate", "Generated");
                    from.into_iter().for_each(|from| self.connect(from, input));
                }
                let from = match chain.iter().find(|(_, bit)| wanted & bit != 0).map(|(name, _)| *name) {
                    Some("LINK_TEXTURE_NORMAL") => self.source(&mut made.texco, "texture_coordinate", "Normal"),
                    Some("LINK_TEXTURE_UV") => self.source(&mut made.texco, "texture_coordinate", "UV"),
                    Some("LINK_TEXTURE_INCOMING") => self.incoming(&mut made),
                    Some("LINK_INCOMING") => self.source(&mut made.geom, "geometry", "Incoming"),
                    Some("LINK_NORMAL") => self.source(&mut made.geom, "geometry", "Normal"),
                    Some("LINK_POSITION") => self.source(&mut made.geom, "geometry", "Position"),
                    Some("LINK_TANGENT") => self.source(&mut made.geom, "geometry", "Tangent"),
                    _ => None,
                };
                from.into_iter().for_each(|from| self.connect(from, input));
            }
            id += 1;
        }
    }
}

#[cfg(test)]
mod test;
