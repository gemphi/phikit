//! The world's shader graph (`World.*nodetree`): the Background node that
//! lights every render, and the Environment Texture behind it.
//!
//! Blender ref (`node_shader_background.cc`, `node_shader_tex_environment.cc`,
//! `background.h`): the World Output's Surface input takes a Background node
//! whose `Color` × `Strength` is the radiance arriving from every direction.
//! `World.horr/horg/horb` is the 2.7x horizon colour and is *not* what a 2.8+
//! render uses — reading it instead of the graph is why our ambient light was
//! wrong.

use super::nodes::read_cstr;
use crate::blend::file::types::*;

/// The world as it lights the scene.
#[derive(Clone, Debug, PartialEq)]
pub struct BlendWorld {
    /// Background `Color`, folded (white when an image feeds it).
    pub color: [f32; 3],
    /// Background `Strength`.
    pub strength: f32,
    /// Environment Texture image key (see `image.rs`), when one feeds `Color`.
    pub texture: Option<String>,
    /// Mapping node `Rotation` around Z, in radians (the usual HDRI spin).
    pub rotation: f32,
}

impl Default for BlendWorld {
    fn default() -> Self {
        // Blender's default world: a near-black grey at full strength.
        Self { color: [0.05; 3], strength: 1.0, texture: None, rotation: 0.0 }
    }
}

impl BlendWorld {
    /// Radiance arriving from the world (colour × strength).
    pub fn radiance(&self) -> [f32; 3] {
        self.color.map(|c| c * self.strength)
    }
}

impl BlendFile {
    /// The scene's world, read from its node tree.
    pub(crate) fn world(&self, wo: &BlendBlock, dna: &DnaType) -> BlendWorld {
        let nodes = self.tree_nodes(wo, "World", dna);
        let graph = super::eval::BGraph::owned(self, wo, "World", dna);
        let named = |n: &BlendBlock| {
            self.off("bNode", "idname", dna).map(|o| read_cstr(&n.data, o, 64)).unwrap_or_default()
        };
        let mut world = BlendWorld::default();
        let Some(bg) = nodes.iter().copied().find(|n| named(n) == "ShaderNodeBackground") else {
            return world;
        };
        for sock in self.input_sockets(bg, dna) {
            let name = self.socket_name(sock, dna);
            let Some(v) = graph.input(sock, 0) else { continue };
            match name.as_str() {
                "Color" => world.color = [v.rgba[0], v.rgba[1], v.rgba[2]],
                "Strength" => world.strength = v.value(),
                _ => {}
            }
        }
        let env = nodes.iter().copied().find(|n| named(n) == "ShaderNodeTexEnvironment");
        if let Some(env) = env {
            world.texture = self
                .block_at(self.ptr(env, self.off("bNode", "*id", dna)))
                .map(|im| self.image_key(im, dna));
            // An image replaces the socket, so Color is the tint it is
            // multiplied by — the fold already reports white for images.
            world.color = [1.0; 3];
        }
        if let Some(map) = nodes.iter().copied().find(|n| named(n) == "ShaderNodeMapping") {
            world.rotation = self.mapping_spin(map, dna);
        }
        world
    }

    /// A Mapping node's `Rotation` around Z, in radians.
    fn mapping_spin(&self, map: &BlendBlock, dna: &DnaType) -> f32 {
        let Some(off) = self.off("bNodeSocketValueVector", "value[3]", dna) else { return 0.0 };
        for sock in self.input_sockets(map, dna) {
            if self.socket_name(sock, dna) != "Rotation" {
                continue;
            }
            let Some(v) = self.block_at(sock).map(|s| self.ptr(s, self.off("bNodeSocket", "*default_value", dna))) else { continue };
            let Some(val) = self.block_at(v) else { continue };
            if let Some(b) = val.data.get(off + 8..off + 12) {
                return self.read_f32(b);
            }
        }
        0.0
    }
}
