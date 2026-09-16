//! Node graph mutation primitives — `add_node`, `remove_node`, `move_node`.
//!
//! Feature: phiber-blender-space-parity
//! Requirements: 22.5, 22.8

use super::{sockets_for, types::*};

impl BNodeGraph {
    /// Insert a fresh node at `(x, y)` with the default sockets returned by
    /// `sockets_for(kind)`. Assigns and returns the node's id, then
    /// increments `next_id`.
    pub fn add_node(&mut self, kind: BNodeKind, x: f32, y: f32) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        let (inputs, outputs) = sockets_for(kind);
        self.nodes.push(BNode { id, kind, x, y, inputs, outputs });
        id
    }

    /// Delete the node with `id` and every edge that touches it.
    ///
    /// Silently no-ops when no such node exists — the outliner and node
    /// graph both call `remove_node` from user-driven flows where a stale
    /// id is not an error condition.
    pub fn remove_node(&mut self, id: u32) {
        self.nodes.retain(|n| n.id != id);
        self.edges.retain(|e| e.from.0 != id && e.to.0 != id);
    }

    /// Update a node's `(x, y)` position. No-op if `id` is absent.
    pub fn move_node(&mut self, id: u32, x: f32, y: f32) {
        if let Some(n) = self.nodes.iter_mut().find(|n| n.id == id) {
            n.x = x;
            n.y = y;
        }
    }
}
