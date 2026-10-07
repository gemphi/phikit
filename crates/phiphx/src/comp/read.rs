//! The scene's compositor tree out of a `.blend`.
//!
//! The tree is a `bNodeTree` like a material's, so `shade::graph`'s
//! `node_graph_at` reads the nodes, sockets and links as they are. What is
//! *not* like a material's is where the pointer to it lives, and that moved
//! in 5.0:
//!
//! ```c
//! /* DNA_scene_types.h:2829 */
//! DNA_DEPRECATED char use_nodes = 0;
//! DNA_DEPRECATED struct bNodeTree *nodetree = nullptr;
//! struct bNodeTree *compositing_node_group = nullptr;
//! ```
//!
//! 5.x writes `compositing_node_group` — a node group datablock the scene
//! points at. Everything saved before it writes `nodetree`, an embedded
//! tree, with `use_nodes` as the switch. Both fields are still in the struct
//! side by side, which is the tell: a reader that takes only the new
//! spelling finds no compositor in any file made before 5.0, and one that
//! takes only the old finds none in a file made after. Each is tried against
//! the file's own DNA.
//!
//! Whether the tree *runs* is a third field again: `Scene.r.scemode`'s
//! `R_DOCOMP` bit (`DNA_scene_types.h:769`), which is Output ▸ Post
//! Processing ▸ Compositing. It is on in a new scene, so a file that has
//! never been touched there still composites.

use super::node::BCompKind;
use super::tree::BCompTree;
use super::node::BCompSet;
use crate::blend::{BlendFile, DnaType};

/// `R_DOCOMP` (`DNA_scene_types.h:769`): Post Processing ▸ Compositing.
pub const R_DOCOMP: i32 = 1 << 6;

impl BlendFile {
    /// The scene's compositor tree, or `None` when the file has none.
    ///
    /// The tree comes back whether or not it would run — `BCompTree::on`
    /// says that — because the Compositing editor draws a tree that is
    /// switched off, and the panel has to be able to switch it back on.
    pub fn compositor(&self) -> Option<BCompTree> {
        let dna = self.dna.as_ref()?;
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let scene = self
            .file_global_curscene(dna)
            .or_else(|| self.blocks.iter().find(|b| b.code_str == "SC"))?;
        let at = |field: &str| {
            Self::struct_field_offset_exact("Scene", field, dna, ps)
                .map(|o| Self::read_ptr(&scene.data, o, ps, le))
                .filter(|&p| p != 0)
        };
        // 5.x first, then the embedded tree every earlier version wrote.
        let (addr, embedded) = match at("*compositing_node_group") {
            Some(a) => (a, false),
            None => (at("*nodetree")?, true),
        };
        let graph = self.node_graph_at(addr, dna)?;
        // `use_nodes` is the switch only for the embedded tree; a 5.x scene
        // that points at a group has no such field to read.
        let used = !embedded
            || Self::struct_field_offset_exact("Scene", "use_nodes", dna, ps)
                .and_then(|o| scene.data.get(o))
                .is_some_and(|b| *b != 0);
        let scemode = Self::struct_field_offset_exact("Scene", "r", dna, ps)
            .and_then(|r| Self::struct_field_offset_exact("RenderData", "scemode", dna, ps).map(|s| r + s))
            .and_then(|o| scene.data.get(o..o + 4))
            .map_or(R_DOCOMP, |b| self.read_i32(b));
        let output = BCompTree::output_of(&graph);
        let setting = self.settings_of(&graph, dna);
        Some(BCompTree { on: used && scemode & R_DOCOMP != 0, output, graph, setting })
    }

    /// What each node keeps in `bNode.storage` rather than on a socket.
    ///
    /// `ShaderNodeMix` — the node a compositor Mix became in 5.x — keeps
    /// its blend mode there, not in `custom1`:
    ///
    /// ```c
    /// /* DNA_node_types.h:3816 */
    /// struct NodeShaderMix {
    ///   eNodeSocketDatatype data_type;
    ///   NodeShaderMixMode factor_mode;
    ///   int8_t clamp_factor;
    ///   int8_t clamp_result;
    ///   int8_t blend_type;
    /// };
    /// ```
    ///
    /// A reader that took `custom1` for the mode read 0 — plain Mix — for
    /// every `ShaderNodeMix` in every file, so a Screen or a Multiply
    /// composited as a straight lerp and looked like a maths error rather
    /// than a missing field. `CompositorNodeMixRGB` in an older file does
    /// keep it in `custom1`, with Clamp Result in `custom2`'s bit 1, so
    /// both are read and the storage wins where there is storage.
    fn settings_of(&self, graph: &crate::blend::shade::graph::BlendNodeGraph, dna: &DnaType) -> Vec<(String, BCompSet)> {
        let ps = self.header.pointer_size as usize;
        let i8_at = |block: &crate::blend::BlendBlock, field: &str| {
            Self::struct_field_offset_exact("NodeShaderMix", field, dna, ps)
                .and_then(|o| block.data.get(o))
                .map(|b| i32::from(*b as i8))
        };
        graph
            .nodes
            .iter()
            .map(|node| {
                let mut set = BCompSet { blend: node.custom1, clamp: node.custom2 & 2 != 0 };
                if node.idname == "ShaderNodeMix"
                    && let Some(block) = (node.storage != 0).then(|| self.block_at(node.storage)).flatten()
                {
                    set.blend = i8_at(block, "blend_type").unwrap_or(set.blend);
                    set.clamp = i8_at(block, "clamp_result").unwrap_or(0) != 0;
                }
                (node.name.clone(), set)
            })
            .collect()
    }
}

impl BCompTree {
    /// Which node's input is the result.
    ///
    /// 5.x ends a compositing tree at the node group's own output
    /// (`NodeGroupOutput`) — the Composite node does not exist any more and
    /// `tree.nodes.new("CompositorNodeComposite")` raises *"Node type
    /// CompositorNodeComposite undefined"*. Earlier files end at a
    /// Composite node, of which there may be several with one marked
    /// `NODE_DO_OUTPUT`. A Viewer is the last resort: a tree with only a
    /// Viewer on the end is what the Image editor's Viewer Node pass shows,
    /// and a reader that insisted on a Composite found no output in it.
    pub(super) fn output_of(graph: &crate::blend::shade::graph::BlendNodeGraph) -> Option<String> {
        let kind = |n: &crate::blend::shade::graph::BlendNode| BCompKind::of(&n.idname);
        let composite = |n: &&crate::blend::shade::graph::BlendNode| kind(n) == BCompKind::Composite;
        graph
            .nodes
            .iter()
            .find(|n| composite(n) && n.active_output())
            .or_else(|| graph.nodes.iter().find(composite))
            .or_else(|| graph.nodes.iter().find(|n| kind(n) == BCompKind::GroupOutput))
            .or_else(|| graph.nodes.iter().find(|n| kind(n) == BCompKind::Viewer))
            .map(|n| n.name.clone())
    }
}
