//! Editing a socket's value: the overlay of socket default-values the user has
//! changed, merged over the file's own tree.
//!
//! The file is read-only — phiber reads the node tree out of the `.blend` on
//! every frame (`link/mod.rs` says why) — so a changed socket value is an
//! overlay, exactly as a made link (`link/`) or an added node (`add/`) is.
//! `apply` overrides the matching input socket's default on the cloned graph;
//! nothing is written back until `phiphx` grows a node-tree writer, at which
//! point this layer feeds it and becomes the undo buffer.
//!
//! Only INPUT sockets carry a drawn value (`socket::value::shows_value`), so
//! only inputs are edited. One entry per `(node, socket)` — setting a value
//! twice replaces, it does not stack — which keeps the whole of a tree's value
//! edits in one store write and one undo step, as the link and add overlays do.

pub mod hit;
pub mod layer;
pub mod store;
#[cfg(test)]
mod tests;

use crate::phiphx::blend::shade::graph::BlendNode;
use crate::phiphx::blend::shade::socket::BSocketDefault;

pub use store::{decode, edits_of, encode};

/// One socket whose value the user has changed, named by the node and socket it
/// belongs to (the same `(name, identifier)` a link resolves by) and the value
/// to show in place of the file's.
#[derive(Clone, Debug, PartialEq)]
pub struct BValueEdit {
    pub node: String,
    pub socket: String,
    pub value: BSocketDefault,
}

/// The socket values changed on top of the file's own, for one host.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BValueEdits {
    pub edits: Vec<BValueEdit>,
}

impl BValueEdits {
    /// Override each edited input socket's default on `nodes` — the merged graph
    /// `add::apply` produced, cloned from the read-only file. An edit naming a
    /// node or socket that is not there is dropped: the file may have changed
    /// under an overlay that outlived it.
    pub fn apply(&self, nodes: &mut [BlendNode]) {
        for e in &self.edits {
            let Some(node) = nodes.iter_mut().find(|n| n.name == e.node) else { continue };
            let Some(sock) = node.inputs.iter_mut().find(|s| s.identifier == e.socket) else { continue };
            sock.default = Some(e.value.clone());
        }
    }

    /// The value this overlay currently sets for a socket, if any — what a
    /// toggle or a step reads to know where it starts before the file is asked.
    pub fn get(&self, node: &str, socket: &str) -> Option<&BSocketDefault> {
        self.edits.iter().find(|e| e.node == node && e.socket == socket).map(|e| &e.value)
    }

    /// Set a socket's value, replacing any value already set for it — one entry
    /// per `(node, socket)`, so a value does not stack with its own past.
    pub fn set(&mut self, node: &str, socket: &str, value: BSocketDefault) {
        self.edits.retain(|e| !(e.node == node && e.socket == socket));
        self.edits.push(BValueEdit { node: node.to_string(), socket: socket.to_string(), value });
    }
}
