//! What a node editor stores about its own tree — selection and where the
//! user has dragged each node, under `ctx.<host>.node.*`.
//!
//! Scoped by host on purpose. Blender gives every `SpaceNode` its own
//! `edittree` (`DNA_space_types.h`: `SpaceNode.edittree`), and the Shader,
//! Compositor and Geometry Nodes editors are three different trees of three
//! different kinds. Selecting a node in the Shader editor must not move the
//! Compositor's selection with it, so the host is part of the key rather than
//! a filter applied afterwards.
//!
//! The node's own `name` is the id, because `bNode.name` is unique within a
//! tree (`BKE_node.hh`, `node_unique_name`) and is what `bNodeLink` resolves
//! its ends to. It is interned (`state::intern`), the same way the Outliner's
//! rows and the animation channels are, since a node is named by the file and
//! no name is known at compile time.

use super::BKey;
use crate::v2::host::BHostType;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeKey {
    /// `ctx.<host>.node.selected` — the node `NODE_SELECT` would mark.
    ///
    /// One name, so one node. Blender's selection is a set and a box select or
    /// a Shift-click grows it; phiber has no multi-selection yet, and the drag
    /// here moves the grabbed node alone for the same reason — see
    /// `canvas/node/drag.rs`.
    Selected(BHostType),
    /// `ctx.<host>.node.at.<name>` — how far the user has dragged this node
    /// from where the file put it, as `"dx, dy"` in canvas fractions.
    ///
    /// A displacement, not a position: `bNode.location` out of the file stays
    /// the truth, and a node with no entry here sits exactly where the file
    /// put it. Absolute positions would need the whole tree seeded into state
    /// on every file load and would silently outlive the file they came from.
    At(BHostType, &'static str),
    /// `ctx.<host>.node.wired` — the links the user has made and broken, over
    /// the ones the file carries.
    ///
    /// One key holding the whole edit list rather than a key per link, because
    /// a link has no id: `bNodeLink` is identified by its two ends and nothing
    /// else, so there is no stable name to put in a path. The value is a
    /// semicolon-separated list of `+from/sock>to/sock` and `-from/sock>to/sock`
    /// (`canvas/node/link/mod.rs` encodes it), which keeps the whole rewiring
    /// of a tree in one store write and one undo step — which is also what
    /// Blender's link operator does, since it commits its whole `nldrag.links`
    /// at once.
    Wired(BHostType),
    /// `ctx.<host>.node.linking` — a link drag in progress.
    ///
    /// Blender keeps this in the operator's own `bNodeLinkDrag` for the life of
    /// the modal, which is the right place for it there. A socket here is
    /// PAINTED rather than an element, so there is no div under one to hang a
    /// GPUI drag on, and the gesture is driven from the canvas's mouse events
    /// instead — which needs somewhere between the press and the release to
    /// keep the anchored end and the cursor. That is this key, and it is
    /// cleared on release either way (`canvas/node/link/live/mod.rs`).
    Linking(BHostType),
}

impl BKey for NodeKey {
    fn path(&self) -> BPath {
        match self {
            Self::Selected(host) => BPath::owned(vec![host.slug(), "node", "selected"]),
            Self::At(host, name) => BPath::owned(vec![host.slug(), "node", "at", name]),
            Self::Wired(host) => BPath::owned(vec![host.slug(), "node", "wired"]),
            Self::Linking(host) => BPath::owned(vec![host.slug(), "node", "linking"]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_nodes_keys_carry_the_editor_they_belong_to() {
        // Blender's three node editors are three `edittree`s; a key that
        // dropped the host would make one selection serve all of them.
        let shader = NodeKey::Selected(BHostType::Shader).path().encode();
        let comp = NodeKey::Selected(BHostType::Compositor).path().encode();
        assert_ne!(shader, comp, "two node editors must not share a selection");
        assert!(shader.ends_with(".node.selected"), "{shader}");
    }

    #[test]
    fn a_dragged_nodes_key_is_named_after_the_node() {
        let a = NodeKey::At(BHostType::Shader, "Principled BSDF").path().encode();
        let b = NodeKey::At(BHostType::Shader, "Material Output").path().encode();
        assert_ne!(a, b, "two nodes must not share one displacement");
        assert_eq!(a, "ctx.shader.node.at.Principled BSDF");
    }
}
