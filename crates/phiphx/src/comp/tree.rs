//! The tree itself: the nodes in the order they have to run, and what runs.

use super::node::{BCompKind, BCompSet};
use crate::blend::shade::graph::{BlendNode, BlendNodeGraph};

/// A scene's compositor tree, read and ready to run.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BCompTree {
    /// Output ▸ Post Processing ▸ Compositing, and the tree's own switch
    /// (`use_nodes` where the file has one): whether a render goes through
    /// this tree at all.
    pub on: bool,
    /// The node whose input is the result (`read.rs`'s `output_of`).
    pub output: Option<String>,
    pub graph: BlendNodeGraph,
    /// What each node keeps in `bNode.storage` rather than on a socket,
    /// by node name — dug out while the reader still had the file
    /// (`read.rs`'s `settings_of`).
    pub setting: Vec<(String, BCompSet)>,
}

/// One step of the run: a node and, per input socket, where its pixel comes
/// from.
pub(super) struct BCompStep<'a> {
    pub node: &'a BlendNode,
    pub kind: BCompKind,
}

impl BCompTree {
    /// Whether this tree would change a render: it runs, it has an output,
    /// and something reaches that output.
    pub fn runs(&self) -> bool {
        self.on && !self.order().is_empty()
    }

    /// The nodes this pass does not run, by `idname`, in the tree's order
    /// and without repeats — what the panel reports rather than silently
    /// dropping. Only nodes that actually reach the output are counted: a
    /// Blur left unconnected in a corner of the graph changes nothing.
    pub fn declined(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for step in self.order() {
            if !step.kind.runs() && !out.contains(&step.node.idname) {
                out.push(step.node.idname.clone());
            }
        }
        out
    }

    /// Node `name`'s storage settings; all-default when it has none.
    pub(super) fn set(&self, name: &str) -> BCompSet {
        self.setting.iter().find(|(n, _)| n == name).map(|(_, s)| *s).unwrap_or_default()
    }

    /// The node `name`.
    pub(super) fn node(&self, name: &str) -> Option<&BlendNode> {
        self.graph.nodes.iter().find(|n| n.name == name)
    }

    /// What feeds input socket `id` of node `name`: the node on the other
    /// end of the link, with reroutes followed through.
    pub(super) fn feeding(&self, name: &str, id: &str) -> Option<&BlendNode> {
        let mut from = self
            .graph
            .links
            .iter()
            .find(|l| l.to == name && l.to_socket == id)
            .map(|l| l.from.clone())?;
        // A reroute is a wire, not a node: walk through it, with a guard so a
        // reroute loop cannot hang the run.
        for _ in 0..64 {
            let node = self.node(&from)?;
            if BCompKind::of(&node.idname) != BCompKind::Reroute {
                return Some(node);
            }
            from = self.graph.links.iter().find(|l| l.to == node.name)?.from.clone();
        }
        None
    }

    /// The nodes to run, inputs before the nodes that read them, ending at
    /// the output. Depth-first from the output and reversed, so a node that
    /// two others read is run once — Blender's own evaluation order.
    ///
    /// Empty when the tree has no output, or nothing links into it: that is
    /// a tree that does nothing, and a render goes through untouched.
    pub(super) fn order(&self) -> Vec<BCompStep<'_>> {
        let Some(output) = self.output.as_deref().and_then(|n| self.node(n)) else {
            return Vec::new();
        };
        // What feeds the output is found by *socket*, not by name: 5.x's
        // group output carries the result on `Socket_1`, named "Image",
        // and a Composite node carries it on "Image".
        if output
            .inputs
            .iter()
            .filter(|s| s.available() && s.identifier != "__extend__")
            .all(|s| self.feeding(&output.name, &s.identifier).is_none())
        {
            return Vec::new();
        }
        let mut order: Vec<&BlendNode> = Vec::new();
        self.walk(output, &mut order, 0);
        order.into_iter().map(|node| BCompStep { kind: BCompKind::of(&node.idname), node }).collect()
    }

    /// `node`'s inputs, then `node`. `depth` guards a graph with a cycle in
    /// it, which Blender refuses to make but a hand-edited file can hold.
    fn walk<'a>(&'a self, node: &'a BlendNode, order: &mut Vec<&'a BlendNode>, depth: usize) {
        if depth > 256 || order.iter().any(|n| n.name == node.name) {
            return;
        }
        for socket in node.inputs.iter().filter(|s| s.available()) {
            if let Some(from) = self.feeding(&node.name, &socket.identifier) {
                self.walk(from, order, depth + 1);
            }
        }
        order.push(node);
    }
}
