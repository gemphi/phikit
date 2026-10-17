//! Deleting nodes: the overlay of nodes the user has removed, taken off the
//! merged tree before it is drawn.
//!
//! The file is read-only — phiber reads the node tree out of the `.blend` on
//! every frame (`link/mod.rs` says why) — so a deleted node is an overlay, the
//! mirror of an added one (`add/`): a set of names taken OUT, merged after the
//! additions so one name removes a file node and an added node alike. Nothing
//! is written back until `phiphx` grows a node-tree writer, at which point this
//! layer feeds it and joins the undo buffer.
//!
//! `NODE_OT_delete` (`editors/space_node/node_edit.cc`): the selected nodes and
//! every link touching them come out. The nodes are dropped here (`apply`); the
//! links are dropped where the effective link list is built (`prune`, in
//! `set.rs`), since a wire to a node that is gone is a wire to nowhere — Blender
//! frees those links in `node_free_node` as the node goes.

pub mod op;
#[cfg(test)]
mod tests;

use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeLink};
use crate::v2::host::BHostType;
use crate::v2::state::{BKey as _, BState, NodeKey};

/// `active;also;also` — one entry per removed node, by name.
const NEXT: char = ';';

/// The nodes removed on top of the file's tree, for one host.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BNodeRemovals {
    pub names: Vec<String>,
}

impl BNodeRemovals {
    pub fn of(names: &[&str]) -> Self {
        Self { names: names.iter().map(|n| n.to_string()).collect() }
    }

    pub fn holds(&self, name: &str) -> bool {
        self.names.iter().any(|n| n == name)
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Add `name` to the set, if it is not already in it.
    pub fn add(&mut self, name: &str) {
        if !self.holds(name) {
            self.names.push(name.to_string());
        }
    }

    /// Drop every removed node from the merged list — the file's nodes with the
    /// additions already merged in, so a removed addition goes too.
    pub fn apply(&self, nodes: &mut Vec<BlendNode>) {
        nodes.retain(|n| !self.holds(&n.name));
    }

    /// Drop every link with either end on a removed node: a wire to a node that
    /// is gone is a wire to nowhere.
    pub fn prune(&self, links: &mut Vec<BlendNodeLink>) {
        links.retain(|l| !self.holds(&l.from) && !self.holds(&l.to));
    }
}

pub fn encode(removals: &BNodeRemovals) -> String {
    removals.names.join(&NEXT.to_string())
}

pub fn decode(text: &str) -> BNodeRemovals {
    BNodeRemovals {
        names: text.split(NEXT).filter(|n| !n.is_empty()).map(str::to_string).collect(),
    }
}

/// This host's removals, out of the store.
pub fn removed_of(state: &BState, host: BHostType) -> BNodeRemovals {
    decode(state.get(&NodeKey::Removed(host).path()).unwrap_or(""))
}
